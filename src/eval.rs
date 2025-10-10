use crate::builtins;
use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::{Env, Expr, FStringPart, Function, Macro, MatchArm, NativeFunc, Pattern, Value};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// モジュール情報
#[derive(Debug, Clone)]
struct Module {
    #[allow(dead_code)]
    name: String,
    exports: HashMap<String, Value>,
}

#[derive(Clone)]
pub struct Evaluator {
    global_env: Arc<RwLock<Env>>,
    defer_stack: Arc<RwLock<Vec<Vec<Expr>>>>, // スコープごとのdeferスタック（LIFO）
    modules: Arc<RwLock<HashMap<String, Arc<Module>>>>, // ロード済みモジュール
    current_module: Arc<RwLock<Option<String>>>, // 現在評価中のモジュール名
    loading_modules: Arc<RwLock<Vec<String>>>, // 循環参照検出用
    #[allow(dead_code)]
    call_stack: Arc<RwLock<Vec<String>>>, // 関数呼び出しスタック（スタックトレース用）
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Env::new();
        let env_rc = Arc::new(RwLock::new(env));

        // 組み込み関数を登録
        builtins::register_all(&env_rc);

        // 特殊な関数を登録（printとlist）
        env_rc.write().set(
            "print".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "print".to_string(),
                func: native_print,
            }),
        );
        env_rc.write().set(
            "list".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "list".to_string(),
                func: native_list,
            }),
        );

        // 型判定関数（builtins以外のもの）
        env_rc.write().set(
            "number?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "number?".to_string(),
                func: native_is_number,
            }),
        );
        env_rc.write().set(
            "fn?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "fn?".to_string(),
                func: native_is_fn,
            }),
        );

        Evaluator {
            global_env: env_rc,
            defer_stack: Arc::new(RwLock::new(Vec::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
            current_module: Arc::new(RwLock::new(None)),
            loading_modules: Arc::new(RwLock::new(Vec::new())),
            call_stack: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn eval(&self, expr: &Expr) -> Result<Value, String> {
        self.eval_with_env(expr, self.global_env.clone())
    }

    fn eval_with_env(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::FString(parts) => self.eval_fstring(parts, env.clone()),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),

            Expr::Symbol(name) => env
                .read()
                .get(name)
                .ok_or_else(|| fmt_msg(MsgKey::UndefinedVar, &[name])),

            Expr::List(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                Ok(Value::List(values?))
            }

            Expr::Vector(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                Ok(Value::Vector(values?))
            }

            Expr::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match self.eval_with_env(k, env.clone())? {
                        Value::Keyword(k) => k,
                        Value::String(s) => s,
                        Value::Symbol(s) => s,
                        _ => return Err(msg(MsgKey::KeyMustBeKeyword).to_string()),
                    };
                    let value = self.eval_with_env(v, env.clone())?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
            }

            Expr::Def(name, value) => {
                let val = self.eval_with_env(value, env.clone())?;
                // 現在の環境に定義（モジュール内ならmodule_env、通常ならglobal_env）
                env.write().set(name.clone(), val.clone());
                Ok(val)
            }

            Expr::Fn {
                params,
                body,
                is_variadic,
            } => Ok(Value::Function(Arc::new(Function {
                params: params.clone(),
                body: (**body).clone(),
                env: env.read().clone(),
                is_variadic: *is_variadic,
            }))),

            Expr::Let { bindings, body } => {
                let mut new_env = Env::with_parent(env.clone());
                for (name, expr) in bindings {
                    let value = self.eval_with_env(expr, Arc::new(RwLock::new(new_env.clone())))?;
                    new_env.set(name.clone(), value);
                }
                self.eval_with_env(body, Arc::new(RwLock::new(new_env)))
            }

            Expr::If {
                test,
                then,
                otherwise,
            } => {
                let test_val = self.eval_with_env(test, env.clone())?;
                if test_val.is_truthy() {
                    self.eval_with_env(then, env)
                } else if let Some(otherwise) = otherwise {
                    self.eval_with_env(otherwise, env)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::Do(exprs) => {
                // deferスコープを作成
                self.defer_stack.write().push(Vec::new());

                let mut result = Value::Nil;
                for expr in exprs {
                    result = self.eval_with_env(expr, env.clone())?;
                }

                // deferを実行（LIFO順）
                if let Some(defers) = self.defer_stack.write().pop() {
                    for defer_expr in defers.iter().rev() {
                        // deferの評価エラーは無視（元の結果を返す）
                        let _ = self.eval_with_env(defer_expr, env.clone());
                    }
                }

                Ok(result)
            }

            Expr::Match { expr, arms } => {
                let value = self.eval_with_env(expr, env.clone())?;
                self.eval_match(&value, arms, env)
            }

            Expr::Try(expr) => {
                // Tryもdeferスコープを作成
                self.defer_stack.write().push(Vec::new());

                let result = match self.eval_with_env(expr, env.clone()) {
                    Ok(value) => {
                        // {:ok value}
                        let mut map = HashMap::new();
                        map.insert("ok".to_string(), value);
                        Ok(Value::Map(map))
                    }
                    Err(e) => {
                        // {:error e}
                        let mut map = HashMap::new();
                        map.insert("error".to_string(), Value::String(e));
                        Ok(Value::Map(map))
                    }
                };

                // deferを実行（LIFO順、エラーでも必ず実行）
                if let Some(defers) = self.defer_stack.write().pop() {
                    for defer_expr in defers.iter().rev() {
                        let _ = self.eval_with_env(defer_expr, env.clone());
                    }
                }

                result
            }

            Expr::Defer(expr) => {
                // defer式をスタックに追加（評価はしない）
                let mut stack = self.defer_stack.write();
                if let Some(current_scope) = stack.last_mut() {
                    current_scope.push(expr.as_ref().clone());
                } else {
                    // スコープがない場合は新しいスコープを作成
                    stack.push(vec![expr.as_ref().clone()]);
                }
                Ok(Value::Nil)
            }

            Expr::Loop { bindings, body } => self.eval_loop(bindings, body, env),

            Expr::Recur(args) => {
                // 引数を評価
                let values: Result<Vec<_>, _> = args
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                let values = values?;

                // Recurは特別なエラーとして扱う（Valueとして返すことができないため）
                Err(format!("__RECUR__:{}", values.len()))
            }

            // マクロ
            Expr::Mac {
                name,
                params,
                is_variadic,
                body,
            } => {
                let mac = Macro {
                    name: name.clone(),
                    params: params.clone(),
                    body: (**body).clone(),
                    env: env.read().clone(),
                    is_variadic: *is_variadic,
                };
                env.write()
                    .set(name.clone(), Value::Macro(Arc::new(mac)));
                Ok(Value::Symbol(name.clone()))
            }

            Expr::Quasiquote(expr) => self.eval_quasiquote(expr, env, 0),

            Expr::Unquote(_) => Err(msg(MsgKey::UnquoteOutsideQuasiquote).to_string()),

            Expr::UnquoteSplice(_) => {
                Err(msg(MsgKey::UnquoteSpliceOutsideQuasiquote).to_string())
            }

            // モジュールシステム
            Expr::Module(name) => {
                *self.current_module.write() = Some(name.clone());
                Ok(Value::Nil)
            }

            Expr::Export(symbols) => {
                // 現在のモジュール名を取得
                let module_name = self.current_module.read().clone()
                    .ok_or_else(|| msg(MsgKey::ExportOnlyInModule).to_string())?;

                // エクスポートする値を収集
                let mut exports = HashMap::new();
                for symbol in symbols {
                    if let Some(value) = env.read().get(symbol) {
                        exports.insert(symbol.clone(), value);
                    } else {
                        return Err(fmt_msg(MsgKey::SymbolNotFound, &[symbol, &module_name]));
                    }
                }

                // モジュールを登録
                let module = Module {
                    name: module_name.clone(),
                    exports,
                };
                self.modules.write().insert(module_name, Arc::new(module));

                Ok(Value::Nil)
            }

            Expr::Use { module, mode } => {
                self.eval_use(module, mode, env)
            }

            Expr::Call { func, args } => {
                // 高階関数と論理演算子、quoteの特別処理
                if let Expr::Symbol(name) = func.as_ref() {
                    match name.as_str() {
                        "_railway-pipe" => return self.eval_railway_pipe(args, env),
                        "time" => return self.eval_time(args, env),
                        "map" => return self.eval_map(args, env),
                        "filter" => return self.eval_filter(args, env),
                        "reduce" => return self.eval_reduce(args, env),
                        "pmap" => return self.eval_pmap(args, env),
                        "pfilter" => return self.eval_pfilter(args, env),
                        "preduce" => return self.eval_preduce(args, env),
                        "partition" => return self.eval_partition(args, env),
                        "group-by" => return self.eval_group_by(args, env),
                        "map-lines" => return self.eval_map_lines(args, env),
                        "take-while" => return self.eval_take_while(args, env),
                        "drop-while" => return self.eval_drop_while(args, env),
                        "find" => return self.eval_find(args, env),
                        "find-index" => return self.eval_find_index(args, env),
                        "every?" => return self.eval_every(args, env),
                        "some?" => return self.eval_some(args, env),
                        "update-keys" => return self.eval_update_keys(args, env),
                        "update-vals" => return self.eval_update_vals(args, env),
                        "partition-by" => return self.eval_partition_by(args, env),
                        "keep" => return self.eval_keep(args, env),
                        "drop-last" => return self.eval_drop_last(args, env),
                        "split-at" => return self.eval_split_at(args, env),
                        "update" => return self.eval_update(args, env),
                        "update-in" => return self.eval_update_in(args, env),
                        "comp" => return self.eval_comp(args, env),
                        "apply" => return self.eval_apply(args, env),
                        "sort-by" => return self.eval_sort_by(args, env),
                        "chunk" => return self.eval_chunk(args, env),
                        "count-by" => return self.eval_count_by(args, env),
                        "max-by" => return self.eval_max_by(args, env),
                        "min-by" => return self.eval_min_by(args, env),
                        "sum-by" => return self.eval_sum_by(args, env),
                        "swap!" => return self.eval_swap(args, env),
                        "eval" => return self.eval_eval(args, env),
                        "go" => return self.eval_go(args, env),
                        "pipeline" => return self.eval_pipeline(args, env),
                        "pipeline-map" => return self.eval_pipeline_map(args, env),
                        "pipeline-filter" => return self.eval_pipeline_filter(args, env),
                        "then" => return self.eval_then(args, env),
                        "catch" => return self.eval_catch(args, env),
                        "select!" => return self.eval_select(args, env),
                        "scope-go" => return self.eval_scope_go(args, env),
                        "with-scope" => return self.eval_with_scope(args, env),
                        "and" => return self.eval_and(args, env),
                        "or" => return self.eval_or(args, env),
                        "quote" => return self.eval_quote(args),
                        _ => {}
                    }
                }

                let func_val = self.eval_with_env(func, env.clone())?;

                // マクロの場合は展開してから評価
                if let Value::Macro(mac) = &func_val {
                    let expanded = self.expand_macro(&mac, args, env.clone())?;
                    return self.eval_with_env(&expanded, env);
                }

                let arg_vals: Result<Vec<_>, _> = args
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                let arg_vals = arg_vals?;

                match func_val {
                    Value::NativeFunc(nf) => (nf.func)(&arg_vals),
                    Value::Function(_) => {
                        // apply_funcを使って関数を適用（complementやjuxtの特殊処理を含む）
                        self.apply_func(&func_val, arg_vals)
                    }
                    Value::Keyword(key) => {
                        // キーワードを関数として使う: (:name map) => (get map :name)
                        if arg_vals.len() != 1 {
                            return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["keyword", "1"]));
                        }
                        match &arg_vals[0] {
                            Value::Map(m) => m
                                .get(&key)
                                .cloned()
                                .ok_or_else(|| fmt_msg(MsgKey::KeyNotFound, &[&key])),
                            _ => Err(fmt_msg(MsgKey::TypeOnly, &["keyword fn", "maps"])),
                        }
                    }
                    _ => Err(fmt_msg(MsgKey::NotAFunction, &[&format!("{:?}", func_val)])),
                }
            }
        }
    }

    fn eval_match(
        &self,
        value: &Value,
        arms: &[MatchArm],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        for arm in arms {
            let mut bindings = HashMap::new();
            let mut transforms = Vec::new();

            if self.match_pattern_with_transforms(&arm.pattern, value, &mut bindings, &mut transforms)? {
                // ガード条件のチェック
                if let Some(guard) = &arm.guard {
                    let mut guard_env = Env::with_parent(env.clone());
                    for (name, val) in &bindings {
                        guard_env.set(name.clone(), val.clone());
                    }
                    let guard_val = self.eval_with_env(guard, Arc::new(RwLock::new(guard_env)))?;
                    if !guard_val.is_truthy() {
                        continue;
                    }
                }

                // 変換を適用
                let mut match_env = Env::with_parent(env.clone());
                for (name, val) in bindings {
                    match_env.set(name.clone(), val.clone());
                }

                let match_env_rc = Arc::new(RwLock::new(match_env));

                // 変換を適用して環境を更新
                for (var, transform_expr, original_val) in transforms {
                    let result = self.apply_transform(&transform_expr, &original_val, match_env_rc.clone())?;
                    match_env_rc.write().set(var, result);
                }

                return self.eval_with_env(&arm.body, match_env_rc);
            }
        }
        Err(msg(MsgKey::NoMatchingPattern).to_string())
    }

    fn match_pattern_with_transforms(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
        transforms: &mut Vec<(String, Expr, Value)>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Transform(var, transform) => {
                // 変換情報を記録
                transforms.push((var.clone(), (**transform).clone(), value.clone()));
                bindings.insert(var.clone(), value.clone());
                Ok(true)
            }
            Pattern::Map(pattern_pairs) => {
                if let Value::Map(map) = value {
                    for (key, pat) in pattern_pairs {
                        if let Some(val) = map.get(key) {
                            if !self.match_pattern_with_transforms(pat, val, bindings, transforms)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::As(inner_pattern, var) => {
                if self.match_pattern_with_transforms(inner_pattern, value, bindings, transforms)? {
                    bindings.insert(var.clone(), value.clone());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => {
                // 他のパターンは従来のmatch_patternを使用
                self.match_pattern(pattern, value, bindings)
            }
        }
    }

    fn apply_transform(&self, transform: &Expr, value: &Value, env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // 変換式を評価して値に適用
        // transform が関数の場合: (transform value)
        // transform がシンボルの場合: (symbol value)
        let transform_val = self.eval_with_env(transform, env.clone())?;
        self.apply_function(&transform_val, &[value.clone()])
    }

    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Nil => Ok(matches!(value, Value::Nil)),
            Pattern::Bool(b) => Ok(matches!(value, Value::Bool(vb) if vb == b)),
            Pattern::Integer(n) => Ok(matches!(value, Value::Integer(vn) if vn == n)),
            Pattern::Float(f) => Ok(matches!(value, Value::Float(vf) if (vf - f).abs() < f64::EPSILON)),
            Pattern::String(s) => Ok(matches!(value, Value::String(vs) if vs == s)),
            Pattern::Keyword(k) => Ok(matches!(value, Value::Keyword(vk) if vk == k)),
            Pattern::Var(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Vector(patterns) => {
                if let Value::Vector(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.match_pattern(pat, val, bindings)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::List(patterns, _rest) => {
                if let Value::List(values) = value {
                    if patterns.len() > values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.match_pattern(pat, val, bindings)? {
                            return Ok(false);
                        }
                    }
                    // TODO: ...restのサポート
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Map(pattern_pairs) => {
                if let Value::Map(map) = value {
                    for (key, pat) in pattern_pairs {
                        if let Some(val) = map.get(key) {
                            if !self.match_pattern(pat, val, bindings)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::As(inner_pattern, var) => {
                // 内側のパターンをマッチ
                if self.match_pattern(inner_pattern, value, bindings)? {
                    // マッチ成功したら、値全体も変数に束縛
                    bindings.insert(var.clone(), value.clone());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Transform(_, _) => {
                // Transformは match_pattern_with_transforms で処理される
                unreachable!("Transform pattern should be handled in match_pattern_with_transforms")
            }
        }
    }

    /// map関数の実装: (map f coll)
    fn eval_map(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::map(&[func, coll], self)
    }

    /// filter関数の実装: (filter pred coll)
    fn eval_filter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::filter(&[pred, coll], self)
    }

    /// reduce関数の実装: (reduce f init coll) または (reduce f coll)
    fn eval_reduce(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;

        if args.len() == 3 {
            let init = self.eval_with_env(&args[1], env.clone())?;
            let coll = self.eval_with_env(&args[2], env.clone())?;
            builtins::reduce(&[func, coll, init], self)
        } else {
            let coll = self.eval_with_env(&args[1], env.clone())?;
            builtins::reduce(&[func, coll], self)
        }
    }

    /// swap!関数の実装: (swap! atom f args...)
    fn eval_swap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let atom = self.eval_with_env(&args[0], env.clone())?;
        let func = self.eval_with_env(&args[1], env.clone())?;
        let mut swap_args = vec![atom, func];
        for arg in &args[2..] {
            swap_args.push(self.eval_with_env(arg, env.clone())?);
        }
        builtins::swap(&swap_args, self)
    }

    /// eval関数の実装: (eval expr)
    fn eval_eval(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let expr = self.eval_with_env(&args[0], env.clone())?;
        builtins::eval(&[expr], self)
    }

    /// pmap関数の実装: (pmap f coll)
    fn eval_pmap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::pmap(&[func, coll], self)
    }

    /// pfilter関数の実装: (pfilter pred coll)
    fn eval_pfilter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("pfilter requires 2 arguments".to_string());
        }
        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::pfilter(&[pred, coll], self)
    }

    /// preduce関数の実装: (preduce f init coll)
    fn eval_preduce(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("preduce requires 3 arguments".to_string());
        }
        let func = self.eval_with_env(&args[0], env.clone())?;
        let init = self.eval_with_env(&args[1], env.clone())?;
        let coll = self.eval_with_env(&args[2], env.clone())?;
        builtins::preduce(&[func, init, coll], self)
    }

    fn eval_partition(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::partition(&[func, coll], self)
    }

    fn eval_group_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::group_by(&[func, coll], self)
    }

    fn eval_map_lines(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let text = self.eval_with_env(&args[1], env.clone())?;
        builtins::map_lines(&[func, text], self)
    }

    fn eval_update(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let map = self.eval_with_env(&args[0], env.clone())?;
        let key = self.eval_with_env(&args[1], env.clone())?;
        let func = self.eval_with_env(&args[2], env.clone())?;
        builtins::update(&[map, key, func], self)
    }

    fn eval_update_in(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let map = self.eval_with_env(&args[0], env.clone())?;
        let path = self.eval_with_env(&args[1], env.clone())?;
        let func = self.eval_with_env(&args[2], env.clone())?;
        builtins::update_in(&[map, path, func], self)
    }

    fn eval_comp(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let funcs: Result<Vec<_>, _> = args.iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect();
        builtins::comp(&funcs?, self)
    }

    fn eval_apply(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let list = self.eval_with_env(&args[1], env.clone())?;
        builtins::apply(&[func, list], self)
    }

    fn eval_take_while(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::take_while(&[pred, coll], self)
    }

    fn eval_drop_while(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::drop_while(&[pred, coll], self)
    }

    /// 関数を適用するヘルパー（builtinsモジュールから使用）
    pub fn apply_function(&self, func: &Value, args: &[Value]) -> Result<Value, String> {
        self.apply_func(func, args.to_vec())
    }

    /// 関数を適用するヘルパー（内部用）
    fn apply_func(&self, func: &Value, args: Vec<Value>) -> Result<Value, String> {
        match func {
            Value::NativeFunc(nf) => (nf.func)(&args),
            Value::Function(f) => {
                // complement特殊処理 - 実行前にチェック
                if let Some(complement_func) = f.env.get("__complement_func__") {
                    let result = self.apply_func(&complement_func, args)?;
                    return Ok(Value::Bool(!result.is_truthy()));
                }

                // juxt特殊処理 - 実行前にチェック
                if let Some(Value::List(juxt_funcs)) = f.env.get("__juxt_funcs__") {
                    let mut results = Vec::new();
                    for jfunc in &juxt_funcs {
                        let result = self.apply_func(jfunc, args.clone())?;
                        results.push(result);
                    }
                    return Ok(Value::Vector(results));
                }

                // 通常の関数処理
                let parent_env = Arc::new(RwLock::new(f.env.clone()));
                let mut new_env = Env::with_parent(parent_env);

                if f.is_variadic {
                    if f.params.len() != 1 {
                        return Err(fmt_msg(MsgKey::VariadicFnNeedsOneParam, &[]));
                    }
                    new_env.set(f.params[0].clone(), Value::List(args));
                } else {
                    if f.params.len() != args.len() {
                        return Err(fmt_msg(
                            MsgKey::ArgCountMismatch,
                            &[&f.params.len().to_string(), &args.len().to_string()],
                        ));
                    }
                    for (param, arg) in f.params.iter().zip(args.iter()) {
                        new_env.set(param.clone(), arg.clone());
                    }
                }

                self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)))
            }
            _ => Err(fmt_msg(MsgKey::NotAFunction, &[&format!("{:?}", func)])),
        }
    }

    /// and論理演算子（短絡評価）
    fn eval_and(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Bool(true));
        }
        let mut last = Value::Bool(true);
        for arg in args {
            last = self.eval_with_env(arg, env.clone())?;
            if !last.is_truthy() {
                return Ok(last);
            }
        }
        Ok(last)
    }

    /// or論理演算子（短絡評価）
    fn eval_or(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Nil);
        }
        for arg in args {
            let val = self.eval_with_env(arg, env.clone())?;
            if val.is_truthy() {
                return Ok(val);
            }
        }
        Ok(Value::Nil)
    }

    /// quote - 式を評価せずにそのまま返す
    fn eval_quote(&self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(fmt_msg(MsgKey::Need1Arg, &["quote"]));
        }
        self.expr_to_value(&args[0])
    }

    /// sort-by - キー関数でソート
    fn eval_sort_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("sort-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::sort_by(&vals, self)
    }

    /// chunk - 固定サイズでリストを分割
    fn eval_chunk(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("chunk requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::chunk(&vals, self)
    }

    /// count-by - 述語でカウント
    fn eval_count_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("count-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::count_by(&vals, self)
    }

    /// max-by - キー関数で最大値を取得
    fn eval_max_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("max-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::max_by(&vals, self)
    }

    /// min-by - キー関数で最小値を取得
    fn eval_min_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("min-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::min_by(&vals, self)
    }

    /// sum-by - キー関数で合計
    fn eval_sum_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("sum-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::sum_by(&vals, self)
    }

    fn eval_go(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("go requires 1 argument".to_string());
        }
        // 式を評価して値に変換
        let val = self.eval_with_env(&args[0], env)?;
        builtins::go(&[val], self)
    }

    fn eval_pipeline(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("pipeline requires 3 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::pipeline(&vals, self)
    }

    fn eval_pipeline_map(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("pipeline-map requires 3 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::pipeline_map(&vals, self)
    }

    fn eval_pipeline_filter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err("pipeline-filter requires 3 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::pipeline_filter(&vals, self)
    }

    fn eval_railway_pipe(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("_railway-pipe: 2個の引数が必要です".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::railway_pipe(&vals, self)
    }

    fn eval_time(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("time: 1個の引数が必要です".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::time(&vals, self)
    }

    fn eval_then(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("then requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::then(&vals, self)
    }

    fn eval_catch(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("catch requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::catch(&vals, self)
    }

    fn eval_select(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("select! requires 1 argument".to_string());
        }
        let val = self.eval_with_env(&args[0], env.clone())?;
        builtins::select(&[val], self)
    }

    fn eval_scope_go(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("scope-go requires 2 arguments".to_string());
        }
        let scope = self.eval_with_env(&args[0], env.clone())?;
        let func = self.eval_with_env(&args[1], env.clone())?;
        builtins::scope_go(&[scope, func], self)
    }

    fn eval_with_scope(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("with-scope requires 1 argument".to_string());
        }
        let func = self.eval_with_env(&args[0], env.clone())?;
        builtins::with_scope(&[func], self)
    }

    fn eval_find(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("find requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::find(&vals, self)
    }

    fn eval_find_index(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("find-index requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::find_index(&vals, self)
    }

    fn eval_every(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("every? requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::every(&vals, self)
    }

    fn eval_some(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("some? requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::some(&vals, self)
    }

    fn eval_update_keys(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("update-keys requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::update_keys(&vals, self)
    }

    fn eval_update_vals(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("update-vals requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::update_vals(&vals, self)
    }

    fn eval_partition_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("partition-by requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::partition_by(&vals, self)
    }

    fn eval_keep(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("keep requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::keep(&vals, self)
    }

    fn eval_drop_last(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("drop-last requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::drop_last(&vals, self)
    }

    fn eval_split_at(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("split-at requires 2 arguments".to_string());
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, env.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::split_at(&vals, self)
    }

}

// eval.rs内でのみ必要な特別な組み込み関数
// （print、list、およびbuiltinsモジュールにない型判定関数）

/// 引数の数をチェックするマクロ
macro_rules! check_args {
    ($args:expr, $expected:expr, $func_name:expr) => {
        if $args.len() != $expected {
            return Err(fmt_msg(
                MsgKey::NeedExactlyNArgs,
                &[$func_name, &$expected.to_string()],
            ));
        }
    };
}

/// list - リストを作成
fn native_list(args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(args.to_vec()))
}

/// print - 値を出力
fn native_print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

/// number? - 数値かどうか判定
fn native_is_number(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "number?");
    Ok(Value::Bool(matches!(args[0], Value::Integer(_) | Value::Float(_))))
}

/// fn? - 関数かどうか判定
fn native_is_fn(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "fn?");
    Ok(Value::Bool(matches!(args[0], Value::Function(_) | Value::NativeFunc(_))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval_str(s: &str) -> Result<Value, String> {
        crate::i18n::init(); // i18nシステムを初期化
        let mut evaluator = Evaluator::new();
        let mut parser = Parser::new(s)?;
        let exprs = parser.parse_all()?;
        let mut result = Value::Nil;
        for expr in exprs {
            result = evaluator.eval(&expr)?;
        }
        Ok(result)
    }

    #[test]
    fn test_integers() {
        assert_eq!(eval_str("42").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_add() {
        assert_eq!(eval_str("(+ 1 2 3)").unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(eval_str("(- 10 3)").unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_mul() {
        assert_eq!(eval_str("(* 2 3 4)").unwrap(), Value::Integer(24));
    }

    #[test]
    fn test_nested() {
        assert_eq!(eval_str("(+ (* 2 3) (- 10 5))").unwrap(), Value::Integer(11));
    }

    #[test]
    fn test_if() {
        assert_eq!(eval_str("(if true 1 2)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(if false 1 2)").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_fn() {
        assert_eq!(
            eval_str("((fn [x] (+ x 1)) 5)").unwrap(),
            Value::Integer(6)
        );
    }

    #[test]
    fn test_let() {
        assert_eq!(
            eval_str("(let [x 10 y 20] (+ x y))").unwrap(),
            Value::Integer(30)
        );
    }

    #[test]
    fn test_match_literal() {
        // 値のマッチ
        assert_eq!(
            eval_str("(match 0 0 -> 42 1 -> 99)").unwrap(),
            Value::Integer(42)
        );
        assert_eq!(
            eval_str("(match 1 0 -> 42 1 -> 99)").unwrap(),
            Value::Integer(99)
        );
    }

    #[test]
    fn test_match_var() {
        // 変数のバインディング
        assert_eq!(
            eval_str("(match 10 n -> (+ n 5))").unwrap(),
            Value::Integer(15)
        );
    }

    #[test]
    fn test_match_wildcard() {
        // ワイルドカード
        assert_eq!(
            eval_str("(match 42 0 -> 1 1 -> 2 _ -> 99)").unwrap(),
            Value::Integer(99)
        );
    }

    #[test]
    fn test_match_nil_bool() {
        // nil/boolの区別
        assert_eq!(
            eval_str("(match nil nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval_str("(match false nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(2)
        );
        assert_eq!(
            eval_str("(match true nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_match_vector() {
        // ベクタのマッチ
        assert_eq!(
            eval_str("(match [1 2] [x y] -> (+ x y))").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_match_guard() {
        // ガード条件
        assert_eq!(
            eval_str("(match 5 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval_str("(match -5 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(-1)
        );
        assert_eq!(
            eval_str("(match 0 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_pipe_simple() {
        // 単純なパイプライン: (10 |> inc) は (inc 10) と同じ
        assert_eq!(
            eval_str("(def inc (fn [x] (+ x 1))) (10 |> inc)").unwrap(),
            Value::Integer(11)
        );
    }

    #[test]
    fn test_pipe_chain() {
        // パイプラインのチェーン: (1 |> inc |> inc) は 3
        assert_eq!(
            eval_str("(def inc (fn [x] (+ x 1))) (1 |> inc |> inc)").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_pipe_with_args() {
        // 引数ありの関数: (10 |> (+ 5)) は (+ 5 10) = 15
        assert_eq!(
            eval_str("(10 |> (+ 5))").unwrap(),
            Value::Integer(15)
        );
    }

    #[test]
    fn test_pipe_complex() {
        // 複雑なパイプライン: (1 |> (+ 2) |> (* 3)) は ((* 3 (+ 2 1))) = 9
        assert_eq!(
            eval_str("(1 |> (+ 2) |> (* 3))").unwrap(),
            Value::Integer(9)
        );
    }

    #[test]
    fn test_map() {
        // mapのテスト
        assert_eq!(
            eval_str("(map (fn [x] (* x 2)) [1 2 3])").unwrap(),
            Value::List(vec![Value::Integer(2), Value::Integer(4), Value::Integer(6)])
        );
    }

    #[test]
    fn test_filter() {
        // filterのテスト
        assert_eq!(
            eval_str("(filter (fn [x] (> x 2)) [1 2 3 4 5])").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(4), Value::Integer(5)])
        );
    }

    #[test]
    fn test_reduce() {
        // reduceのテスト（初期値あり）
        assert_eq!(
            eval_str("(reduce + 0 [1 2 3 4])").unwrap(),
            Value::Integer(10)
        );
        // reduceのテスト（初期値なし）
        assert_eq!(
            eval_str("(reduce + [1 2 3 4])").unwrap(),
            Value::Integer(10)
        );
    }

    #[test]
    fn test_cons() {
        // consのテスト
        assert_eq!(
            eval_str("(cons 1 (list 2 3))").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])
        );
        assert_eq!(
            eval_str("(cons 1 nil)").unwrap(),
            Value::List(vec![Value::Integer(1)])
        );
    }

    #[test]
    fn test_conj() {
        // conjのテスト
        assert_eq!(
            eval_str("(conj [1 2] 3 4)").unwrap(),
            Value::Vector(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3), Value::Integer(4)])
        );
        assert_eq!(
            eval_str("(conj (list 1 2) 3)").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_empty() {
        // empty?のテスト
        assert_eq!(eval_str("(empty? [])").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(empty? [1])").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(empty? nil)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_nil_q() {
        // nil?のテスト
        assert_eq!(eval_str("(nil? nil)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(nil? false)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(nil? 0)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_str() {
        // strのテスト
        assert_eq!(
            eval_str("(str \"hello\" \" \" \"world\")").unwrap(),
            Value::String("hello world".to_string())
        );
        assert_eq!(
            eval_str("(str \"count: \" 42)").unwrap(),
            Value::String("count: 42".to_string())
        );
    }

    #[test]
    fn test_and() {
        // andのテスト（短絡評価）
        assert_eq!(eval_str("(and true true)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(and true false)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(and false true)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(and 1 2 3)").unwrap(), Value::Integer(3));
        assert_eq!(eval_str("(and 1 nil 3)").unwrap(), Value::Nil);
    }

    #[test]
    fn test_or() {
        // orのテスト（短絡評価）
        assert_eq!(eval_str("(or false false)").unwrap(), Value::Nil);
        assert_eq!(eval_str("(or false true)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(or true false)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(or nil 2 3)").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_not() {
        // notのテスト
        assert_eq!(eval_str("(not true)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(not false)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(not nil)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(not 42)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_pipeline_with_builtins() {
        // パイプラインと新しい組み込み関数の組み合わせ
        assert_eq!(
            eval_str("[1 2 3 4 5] |> (filter (fn [x] (> x 2))) |> (map (fn [x] (* x 2)))").unwrap(),
            Value::List(vec![Value::Integer(6), Value::Integer(8), Value::Integer(10)])
        );
    }

    #[test]
    fn test_mod() {
        // %（剰余）のテスト
        assert_eq!(eval_str("(% 10 3)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(% 15 4)").unwrap(), Value::Integer(3));
        assert_eq!(eval_str("(% 8 2)").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_le() {
        // <=のテスト
        assert_eq!(eval_str("(<= 5 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(<= 10 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(<= 15 10)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_ge() {
        // >=のテスト
        assert_eq!(eval_str("(>= 10 5)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(>= 10 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(>= 5 10)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_ne() {
        // !=のテスト
        assert_eq!(eval_str("(!= 1 2)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(!= 1 1)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(!= nil false)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_quote() {
        // quoteのテスト
        assert_eq!(
            eval_str("(quote x)").unwrap(),
            Value::Symbol("x".to_string())
        );
        assert_eq!(
            eval_str("'x").unwrap(),
            Value::Symbol("x".to_string())
        );
        assert_eq!(
            eval_str("'(1 2 3)").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])
        );
        assert_eq!(
            eval_str("'(+ 1 2)").unwrap(),
            Value::List(vec![Value::Symbol("+".to_string()), Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_even_with_mod() {
        // %を使った偶数判定
        assert_eq!(
            eval_str("(def even? (fn [x] (= (% x 2) 0))) (even? 4)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval_str("(def even? (fn [x] (= (% x 2) 0))) (even? 5)").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_try_success() {
        // 成功時は {:ok value}
        let result = eval_str("(try (+ 1 2))").unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.get("ok"), Some(&Value::Integer(3)));
                assert_eq!(m.get("error"), None);
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_try_error() {
        // エラー時は {:error msg}
        let result = eval_str("(try (/ 1 0))").unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.get("ok"), None);
                assert!(m.get("error").is_some());
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_try_with_match() {
        // tryとmatchの組み合わせ
        let result = eval_str(
            r#"
            (match (try (+ 1 2))
              {:ok result} -> result
              {:error e} -> 0)
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(3));

        let result = eval_str(
            r#"
            (match (try (/ 1 0))
              {:ok result} -> result
              {:error e} -> -1)
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_defer_basic() {
        // deferはスコープ終了時に実行される
        // deferがnilを返すことを確認
        let result = eval_str(
            r#"
            (do
              (defer (+ 1 2))
              42)
            "#,
        )
        .unwrap();
        // doの結果は42（deferの結果ではない）
        assert_eq!(result, Value::Integer(42));
    }

    // リスト操作関数のテスト

    #[test]
    fn test_nth() {
        assert_eq!(eval_str("(nth [10 20 30] 0)").unwrap(), Value::Integer(10));
        assert_eq!(eval_str("(nth [10 20 30] 1)").unwrap(), Value::Integer(20));
        assert_eq!(eval_str("(nth [10 20 30] 2)").unwrap(), Value::Integer(30));
        assert_eq!(eval_str("(nth [10 20 30] 5)").unwrap(), Value::Nil);
    }

    #[test]
    fn test_count() {
        assert_eq!(eval_str("(count [1 2 3])").unwrap(), Value::Integer(3));
        assert_eq!(eval_str("(count [])").unwrap(), Value::Integer(0));
        assert_eq!(eval_str("(count '(1 2 3 4))").unwrap(), Value::Integer(4));
        assert_eq!(eval_str("(count {:a 1 :b 2})").unwrap(), Value::Integer(2));
        assert_eq!(eval_str("(count \"hello\")").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_reverse() {
        assert_eq!(
            eval_str("(reverse [1 2 3])").unwrap(),
            Value::Vector(vec![Value::Integer(3), Value::Integer(2), Value::Integer(1)])
        );
        assert_eq!(
            eval_str("(reverse '(a b c))").unwrap(),
            Value::List(vec![
                Value::Symbol("c".to_string()),
                Value::Symbol("b".to_string()),
                Value::Symbol("a".to_string())
            ])
        );
    }

    // 型チェック関数のテスト

    #[test]
    fn test_type_predicates() {
        assert_eq!(eval_str("(list? '(1 2 3))").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(list? [1 2 3])").unwrap(), Value::Bool(false));

        assert_eq!(eval_str("(vector? [1 2 3])").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(vector? '(1 2 3))").unwrap(), Value::Bool(false));

        assert_eq!(eval_str("(map? {:a 1})").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(map? [1 2])").unwrap(), Value::Bool(false));

        assert_eq!(eval_str("(string? \"hello\")").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(string? 123)").unwrap(), Value::Bool(false));

        assert_eq!(eval_str("(number? 42)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(number? \"42\")").unwrap(), Value::Bool(false));

        assert_eq!(eval_str("(fn? (fn [] 1))").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(fn? +)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(fn? 42)").unwrap(), Value::Bool(false));
    }

    // 数学関数のテスト

    #[test]
    fn test_abs() {
        assert_eq!(eval_str("(abs 5)").unwrap(), Value::Integer(5));
        assert_eq!(eval_str("(abs -5)").unwrap(), Value::Integer(5));
        assert_eq!(eval_str("(abs 0)").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_min_max() {
        assert_eq!(eval_str("(min 3 1 4 1 5)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(max 3 1 4 1 5)").unwrap(), Value::Integer(5));
        assert_eq!(eval_str("(min 10)").unwrap(), Value::Integer(10));
        assert_eq!(eval_str("(max 10)").unwrap(), Value::Integer(10));
    }
}

// モジュールシステムのヘルパー関数
impl Evaluator {
    /// useモジュールの評価
    fn eval_use(
        &self,
        module_name: &str,
        mode: &crate::value::UseMode,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        use crate::value::UseMode;

        // モジュールをロード
        let module = self.load_module(module_name)?;

        // インポートモードに応じて環境に追加
        match mode {
            UseMode::Only(names) => {
                // 指定された関数のみインポート
                for name in names {
                    if let Some(value) = module.exports.get(name) {
                        env.write().set(name.clone(), value.clone());
                    } else {
                        return Err(fmt_msg(MsgKey::SymbolNotExported, &[name, module_name]));
                    }
                }
            }
            UseMode::All => {
                // 全てインポート
                for (name, value) in &module.exports {
                    env.write().set(name.clone(), value.clone());
                }
            }
            UseMode::As(_alias) => {
                // TODO: エイリアス機能は将来実装
                return Err(msg(MsgKey::UseAsNotImplemented).to_string());
            }
        }

        Ok(Value::Nil)
    }

    /// モジュールファイルをロード
    fn load_module(&self, name: &str) -> Result<Arc<Module>, String> {
        // 既にロード済みならキャッシュから返す
        if let Some(module) = self.modules.read().get(name) {
            return Ok(module.clone());
        }

        // 循環参照チェック
        {
            let loading = self.loading_modules.read();
            if loading.contains(&name.to_string()) {
                return Err(fmt_msg(MsgKey::CircularDependency, &[&loading.join(" -> ")]));
            }
        }

        // ロード中のモジュールリストに追加
        self.loading_modules.write().push(name.to_string());

        // ファイルを探す（カレントディレクトリとexamples/）
        let paths = vec![
            format!("{}.qi", name),
            format!("examples/{}.qi", name),
        ];

        let mut content = None;
        for path in &paths {
            if let Ok(c) = std::fs::read_to_string(path) {
                content = Some(c);
                break;
            }
        }

        let content = content.ok_or_else(|| {
            fmt_msg(MsgKey::ModuleNotFound, &[name])
        })?;

        // パースして評価
        let mut parser = crate::parser::Parser::new(&content)
            .map_err(|e| fmt_msg(MsgKey::ModuleParserInitError, &[name, &e]))?;

        let exprs = parser.parse_all()
            .map_err(|e| fmt_msg(MsgKey::ModuleParseError, &[name, &e]))?;

        // 新しい環境で評価
        let module_env = Arc::new(RwLock::new(Env::new()));

        // グローバル環境から組み込み関数をコピー
        let bindings: Vec<_> = self.global_env.read().bindings()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in bindings {
            module_env.write().set(key, value);
        }

        // 式を順次評価
        for expr in exprs {
            self.eval_with_env(&expr, module_env.clone())?;
        }

        // ロード中リストから削除
        self.loading_modules.write().pop();

        // モジュールが登録されているか確認
        self.modules.read().get(name).cloned()
            .ok_or_else(|| fmt_msg(MsgKey::ModuleMustExport, &[name]))
    }

    /// f-stringを評価
    fn eval_fstring(&self, parts: &[FStringPart], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let mut result = String::new();

        for part in parts {
            match part {
                FStringPart::Text(text) => result.push_str(text),
                FStringPart::Code(code) => {
                    // コードをパースして評価
                    let mut parser = crate::parser::Parser::new(code)
                        .map_err(|e| format!("f-string: コードのパースエラー: {}", e))?;
                    let expr = parser.parse()
                        .map_err(|e| format!("f-string: コードのパースエラー: {}", e))?;
                    let value = self.eval_with_env(&expr, env.clone())?;

                    // 値を文字列に変換
                    let s = match value {
                        Value::String(s) => s,
                        Value::Integer(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Nil => "nil".to_string(),
                        Value::Keyword(k) => format!(":{}", k),
                        Value::Symbol(s) => s,
                        Value::List(items) => {
                            let strs: Vec<_> = items.iter()
                                .map(|v| format!("{}", v))
                                .collect();
                            format!("({})", strs.join(" "))
                        }
                        Value::Vector(items) => {
                            let strs: Vec<_> = items.iter()
                                .map(|v| format!("{}", v))
                                .collect();
                            format!("[{}]", strs.join(" "))
                        }
                        Value::Map(m) => {
                            let strs: Vec<_> = m.iter()
                                .map(|(k, v)| format!(":{} {}", k, v))
                                .collect();
                            format!("{{{}}}", strs.join(" "))
                        }
                        Value::Function(_) => "<function>".to_string(),
                        Value::NativeFunc(nf) => format!("<native-fn:{}>", nf.name),
                        Value::Macro(m) => format!("<macro:{}>", m.name),
                        Value::Atom(a) => format!("<atom:{}>", a.read()),
                        Value::Channel(_) => "<channel>".to_string(),
                        Value::Scope(_) => "<scope>".to_string(),
                        Value::Uvar(id) => format!("<uvar:{}>", id),
                    };
                    result.push_str(&s);
                }
            }
        }

        Ok(Value::String(result))
    }

    /// loopを評価
    fn eval_loop(
        &self,
        bindings: &[(String, Expr)],
        body: &Expr,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        // ループ用の環境を作成
        let mut loop_env = Env::with_parent(env.clone());

        // 初期値で環境を設定
        let mut current_values = Vec::new();
        for (_name, expr) in bindings {
            let value = self.eval_with_env(expr, env.clone())?;
            current_values.push(value);
        }

        // 環境に設定
        for ((name, _), value) in bindings.iter().zip(current_values.iter()) {
            loop_env.set(name.clone(), value.clone());
        }

        let loop_env_rc = Arc::new(RwLock::new(loop_env));

        // ループ本体を繰り返し評価
        loop {
            match self.eval_with_env(body, loop_env_rc.clone()) {
                Ok(value) => return Ok(value),
                Err(e) if e.starts_with("__RECUR__:") => {
                    // Recurエラーを検出 - 評価し直す必要がある
                    // 実際のrecur呼び出しを見つけて引数を評価
                    if let Some(args) = Self::find_recur(body) {
                        let new_values: Result<Vec<_>, _> = args
                            .iter()
                            .map(|e| self.eval_with_env(e, loop_env_rc.clone()))
                            .collect();
                        let new_values = new_values?;

                        // 環境を更新
                        if bindings.len() != new_values.len() {
                            return Err(format!(
                                "recur: 引数の数が一致しません（期待: {}, 実際: {}）",
                                bindings.len(),
                                new_values.len()
                            ));
                        }

                        for ((name, _), value) in bindings.iter().zip(new_values.iter()) {
                            loop_env_rc.write().set(name.clone(), value.clone());
                        }
                    } else {
                        return Err(msg(MsgKey::RecurNotFound).to_string());
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Exprからrecurを見つける（簡易版）
    fn find_recur(expr: &Expr) -> Option<&Vec<Expr>> {
        match expr {
            Expr::Recur(args) => Some(args),
            Expr::If { then, otherwise, .. } => {
                Self::find_recur(then).or_else(|| otherwise.as_ref().and_then(|e| Self::find_recur(e)))
            }
            Expr::Do(exprs) => exprs.iter().find_map(Self::find_recur),
            _ => None,
        }
    }

    /// quasiquoteを評価
    fn eval_quasiquote(&self, expr: &Expr, env: Arc<RwLock<Env>>, depth: usize) -> Result<Value, String> {
        match expr {
            Expr::Unquote(e) if depth == 0 => {
                // depth 0のunquoteは評価
                self.eval_with_env(e, env)
            }
            Expr::Unquote(e) => {
                // ネストしたquasiquote内のunquote
                let inner = self.eval_quasiquote(e, env, depth - 1)?;
                Ok(inner)
            }
            Expr::Quasiquote(e) => {
                // ネストしたquasiquote
                self.eval_quasiquote(e, env, depth + 1)
            }
            Expr::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    if let Expr::UnquoteSplice(e) = item {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(e, env.clone())?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => return Err(msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()),
                            }
                        } else {
                            let val = self.eval_quasiquote(e, env.clone(), depth - 1)?;
                            result.push(val);
                        }
                    } else {
                        let val = self.eval_quasiquote(item, env.clone(), depth)?;
                        result.push(val);
                    }
                }
                Ok(Value::List(result))
            }
            Expr::Vector(items) => {
                let mut result = Vec::new();
                for item in items {
                    let val = self.eval_quasiquote(item, env.clone(), depth)?;
                    result.push(val);
                }
                Ok(Value::Vector(result))
            }
            Expr::Call { func, args } => {
                // Callもリストとして扱う
                let mut result = vec![self.eval_quasiquote(func, env.clone(), depth)?];
                for arg in args {
                    if let Expr::UnquoteSplice(e) = arg {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(e, env.clone())?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => return Err(msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()),
                            }
                        } else {
                            let val = self.eval_quasiquote(e, env.clone(), depth - 1)?;
                            result.push(val);
                        }
                    } else {
                        let val = self.eval_quasiquote(arg, env.clone(), depth)?;
                        result.push(val);
                    }
                }
                Ok(Value::List(result))
            }
            // 特殊形式もリスト形式に変換
            Expr::If { test, then, otherwise } => {
                let mut result = vec![Value::Symbol("if".to_string())];
                result.push(self.eval_quasiquote(test, env.clone(), depth)?);
                result.push(self.eval_quasiquote(then, env.clone(), depth)?);
                if let Some(o) = otherwise {
                    result.push(self.eval_quasiquote(o, env.clone(), depth)?);
                }
                Ok(Value::List(result))
            }
            Expr::Do(exprs) => {
                let mut result = vec![Value::Symbol("do".to_string())];
                for e in exprs {
                    if let Expr::UnquoteSplice(us) = e {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(us, env.clone())?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => return Err(msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()),
                            }
                        } else {
                            result.push(self.eval_quasiquote(us, env.clone(), depth - 1)?);
                        }
                    } else {
                        result.push(self.eval_quasiquote(e, env.clone(), depth)?);
                    }
                }
                Ok(Value::List(result))
            }
            // その他は変換してValueに
            _ => self.expr_to_value(expr),
        }
    }

    /// ExprをValueに変換（データとして扱う）
    fn expr_to_value(&self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Symbol(s) => Ok(Value::Symbol(s.clone())),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),
            Expr::List(items) => {
                let vals: Result<Vec<_>, _> = items.iter().map(|e| self.expr_to_value(e)).collect();
                Ok(Value::List(vals?))
            }
            Expr::Vector(items) => {
                let vals: Result<Vec<_>, _> = items.iter().map(|e| self.expr_to_value(e)).collect();
                Ok(Value::Vector(vals?))
            }
            Expr::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match self.expr_to_value(k)? {
                        Value::Keyword(k) => k,
                        Value::String(s) => s,
                        Value::Symbol(s) => s,
                        _ => return Err(msg(MsgKey::KeyMustBeKeyword).to_string()),
                    };
                    let value = self.expr_to_value(v)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
            }
            // 特殊形式やCallは評価せずにリストとして返す
            Expr::Call { func, args } => {
                let mut items = vec![self.expr_to_value(func)?];
                for arg in args {
                    items.push(self.expr_to_value(arg)?);
                }
                Ok(Value::List(items))
            }
            Expr::If { test, then, otherwise } => {
                let mut items = vec![Value::Symbol("if".to_string())];
                items.push(self.expr_to_value(test)?);
                items.push(self.expr_to_value(then)?);
                if let Some(o) = otherwise {
                    items.push(self.expr_to_value(o)?);
                }
                Ok(Value::List(items))
            }
            Expr::Do(exprs) => {
                let mut items = vec![Value::Symbol("do".to_string())];
                for e in exprs {
                    items.push(self.expr_to_value(e)?);
                }
                Ok(Value::List(items))
            }
            Expr::Def(name, value) => {
                Ok(Value::List(vec![
                    Value::Symbol("def".to_string()),
                    Value::Symbol(name.clone()),
                    self.expr_to_value(value)?,
                ]))
            }
            Expr::Let { bindings, body } => {
                let mut items = vec![Value::Symbol("let".to_string())];
                let mut binding_vec = Vec::new();
                for (name, expr) in bindings {
                    binding_vec.push(Value::Symbol(name.clone()));
                    binding_vec.push(self.expr_to_value(expr)?);
                }
                items.push(Value::Vector(binding_vec));
                items.push(self.expr_to_value(body)?);
                Ok(Value::List(items))
            }
            Expr::Fn { params, body, is_variadic } => {
                let mut items = vec![Value::Symbol("fn".to_string())];
                let param_vals: Vec<Value> = if *is_variadic && params.len() == 1 {
                    vec![Value::Symbol("&".to_string()), Value::Symbol(params[0].clone())]
                } else if *is_variadic {
                    let mut v: Vec<Value> = params[..params.len()-1]
                        .iter()
                        .map(|p| Value::Symbol(p.clone()))
                        .collect();
                    v.push(Value::Symbol("&".to_string()));
                    v.push(Value::Symbol(params[params.len()-1].clone()));
                    v
                } else {
                    params.iter().map(|p| Value::Symbol(p.clone())).collect()
                };
                items.push(Value::Vector(param_vals));
                items.push(self.expr_to_value(body)?);
                Ok(Value::List(items))
            }
            Expr::Quasiquote(e) => {
                Ok(Value::List(vec![
                    Value::Symbol("quasiquote".to_string()),
                    self.expr_to_value(e)?,
                ]))
            }
            Expr::Unquote(e) => {
                Ok(Value::List(vec![
                    Value::Symbol("unquote".to_string()),
                    self.expr_to_value(e)?,
                ]))
            }
            Expr::UnquoteSplice(e) => {
                Ok(Value::List(vec![
                    Value::Symbol("unquote-splice".to_string()),
                    self.expr_to_value(e)?,
                ]))
            }
            // モジュール関連とtry、deferはquoteできない
            Expr::Module(_) | Expr::Export(_) | Expr::Use { .. } | Expr::Try(_) | Expr::Defer(_) | Expr::Loop { .. } | Expr::Recur(_) | Expr::Match { .. } | Expr::Mac { .. } => {
                Err(fmt_msg(MsgKey::CannotQuote, &["module/export/use/try/defer/loop/recur/match/mac"]))
            }
            Expr::FString(_) => {
                Err(msg(MsgKey::FStringCannotBeQuoted).to_string())
            }
        }
    }

    /// マクロを展開
    fn expand_macro(&self, mac: &Macro, args: &[Expr], _env: Arc<RwLock<Env>>) -> Result<Expr, String> {
        // マクロ用の環境を作成
        let parent_env = Arc::new(RwLock::new(mac.env.clone()));
        let mut new_env = Env::with_parent(parent_env);

        if mac.is_variadic {
            // 可変長引数の処理：最後のパラメータが可変引数
            if mac.params.is_empty() {
                return Err(msg(MsgKey::VariadicMacroNeedsParams).to_string());
            }

            let fixed_count = mac.params.len() - 1;

            // 固定引数が足りない場合エラー
            if args.len() < fixed_count {
                return Err(format!(
                    "mac {}: 引数の数が不足しています（最低: {}, 実際: {}）",
                    mac.name,
                    fixed_count,
                    args.len()
                ));
            }

            // 固定引数を設定
            for i in 0..fixed_count {
                let arg_val = self.expr_to_value(&args[i])?;
                new_env.set(mac.params[i].clone(), arg_val);
            }

            // 残りを可変引数として設定
            let rest: Vec<Value> = args[fixed_count..]
                .iter()
                .map(|e| self.expr_to_value(e))
                .collect::<Result<Vec<_>, _>>()?;
            new_env.set(mac.params[fixed_count].clone(), Value::List(rest));
        } else {
            // 通常の引数
            if mac.params.len() != args.len() {
                return Err(format!(
                    "mac {}: 引数の数が一致しません（期待: {}, 実際: {}）",
                    mac.name,
                    mac.params.len(),
                    args.len()
                ));
            }
            for (param, arg) in mac.params.iter().zip(args.iter()) {
                // 引数をそのまま環境に（評価しない）
                let arg_val = self.expr_to_value(arg)?;
                new_env.set(param.clone(), arg_val);
            }
        }

        // マクロ本体を評価
        let new_env_rc = Arc::new(RwLock::new(new_env));
        let result = self.eval_with_env(&mac.body, new_env_rc)?;

        // 結果をExprに変換
        self.value_to_expr(&result)
    }

    /// ValueをExprに変換（マクロ展開の結果をコードとして扱う）
    fn value_to_expr(&self, val: &Value) -> Result<Expr, String> {
        match val {
            Value::Nil => Ok(Expr::Nil),
            Value::Bool(b) => Ok(Expr::Bool(*b)),
            Value::Integer(n) => Ok(Expr::Integer(*n)),
            Value::Float(f) => Ok(Expr::Float(*f)),
            Value::String(s) => Ok(Expr::String(s.clone())),
            Value::Symbol(s) => Ok(Expr::Symbol(s.clone())),
            Value::Keyword(k) => Ok(Expr::Keyword(k.clone())),
            Value::List(items) if items.is_empty() => {
                Ok(Expr::List(vec![]))
            }
            Value::List(items) => {
                // 先頭がシンボルの場合、特殊形式かチェック
                if let Some(Value::Symbol(s)) = items.first() {
                    match s.as_str() {
                        "if" if items.len() >= 3 && items.len() <= 4 => {
                            return Ok(Expr::If {
                                test: Box::new(self.value_to_expr(&items[1])?),
                                then: Box::new(self.value_to_expr(&items[2])?),
                                otherwise: if items.len() == 4 {
                                    Some(Box::new(self.value_to_expr(&items[3])?))
                                } else {
                                    None
                                },
                            });
                        }
                        "do" => {
                            let exprs: Result<Vec<_>, _> = items[1..].iter().map(|v| self.value_to_expr(v)).collect();
                            return Ok(Expr::Do(exprs?));
                        }
                        "def" if items.len() == 3 => {
                            if let Value::Symbol(name) = &items[1] {
                                return Ok(Expr::Def(name.clone(), Box::new(self.value_to_expr(&items[2])?)));
                            }
                        }
                        // quasiquote/unquote/unquote-spliceは展開後には出現しないはず
                        // もし出現した場合は通常のリストとして扱う
                        _ => {}
                    }
                }
                // 通常のリストまたは関数呼び出し
                let exprs: Result<Vec<_>, _> = items.iter().map(|v| self.value_to_expr(v)).collect();
                let exprs = exprs?;

                // 先頭がシンボルの場合はCallに変換（関数呼び出しとして扱う）
                if let Some(Expr::Symbol(_)) = exprs.first() {
                    if exprs.len() == 1 {
                        // 単一のシンボルはそのまま
                        Ok(Expr::List(exprs))
                    } else {
                        // 関数呼び出し
                        Ok(Expr::Call {
                            func: Box::new(exprs[0].clone()),
                            args: exprs[1..].to_vec(),
                        })
                    }
                } else {
                    Ok(Expr::List(exprs))
                }
            }
            Value::Vector(items) => {
                let exprs: Result<Vec<_>, _> = items.iter().map(|v| self.value_to_expr(v)).collect();
                Ok(Expr::Vector(exprs?))
            }
            _ => Err(msg(MsgKey::ValueCannotBeConverted).to_string()),
        }
    }
}
