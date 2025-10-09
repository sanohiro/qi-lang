use crate::builtins;
use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::{Env, Expr, Function, MatchArm, NativeFunc, Pattern, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// モジュール情報
#[derive(Debug, Clone)]
struct Module {
    name: String,
    exports: HashMap<String, Value>,
}

pub struct Evaluator {
    global_env: Rc<RefCell<Env>>,
    defer_stack: Vec<Vec<Expr>>, // スコープごとのdeferスタック（LIFO）
    modules: HashMap<String, Rc<Module>>, // ロード済みモジュール
    current_module: Option<String>, // 現在評価中のモジュール名
    loading_modules: Vec<String>, // 循環参照検出用
    call_stack: Vec<String>, // 関数呼び出しスタック（スタックトレース用）
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Env::new();
        let env_rc = Rc::new(RefCell::new(env));

        // 組み込み関数を登録
        builtins::register_all(&env_rc);

        // 特殊な関数を登録（printとlist）
        env_rc.borrow_mut().set(
            "print".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "print".to_string(),
                func: native_print,
            }),
        );
        env_rc.borrow_mut().set(
            "list".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "list".to_string(),
                func: native_list,
            }),
        );

        // 型判定関数（builtins以外のもの）
        env_rc.borrow_mut().set(
            "number?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "number?".to_string(),
                func: native_is_number,
            }),
        );
        env_rc.borrow_mut().set(
            "fn?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "fn?".to_string(),
                func: native_is_fn,
            }),
        );

        Evaluator {
            global_env: env_rc,
            defer_stack: Vec::new(),
            modules: HashMap::new(),
            current_module: None,
            loading_modules: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        self.eval_with_env(expr, self.global_env.clone())
    }

    fn eval_with_env(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, String> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),

            Expr::Symbol(name) => env
                .borrow()
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
                env.borrow_mut().set(name.clone(), val.clone());
                Ok(val)
            }

            Expr::Fn {
                params,
                body,
                is_variadic,
            } => Ok(Value::Function(Rc::new(Function {
                params: params.clone(),
                body: (**body).clone(),
                env: env.borrow().clone(),
                is_variadic: *is_variadic,
            }))),

            Expr::Let { bindings, body } => {
                let mut new_env = Env::with_parent(env.clone());
                for (name, expr) in bindings {
                    let value = self.eval_with_env(expr, Rc::new(RefCell::new(new_env.clone())))?;
                    new_env.set(name.clone(), value);
                }
                self.eval_with_env(body, Rc::new(RefCell::new(new_env)))
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
                self.defer_stack.push(Vec::new());

                let mut result = Value::Nil;
                for expr in exprs {
                    result = self.eval_with_env(expr, env.clone())?;
                }

                // deferを実行（LIFO順）
                if let Some(defers) = self.defer_stack.pop() {
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
                self.defer_stack.push(Vec::new());

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
                if let Some(defers) = self.defer_stack.pop() {
                    for defer_expr in defers.iter().rev() {
                        let _ = self.eval_with_env(defer_expr, env.clone());
                    }
                }

                result
            }

            Expr::Defer(expr) => {
                // defer式をスタックに追加（評価はしない）
                if let Some(current_scope) = self.defer_stack.last_mut() {
                    current_scope.push(expr.as_ref().clone());
                } else {
                    // スコープがない場合は新しいスコープを作成
                    self.defer_stack.push(vec![expr.as_ref().clone()]);
                }
                Ok(Value::Nil)
            }

            // モジュールシステム
            Expr::Module(name) => {
                self.current_module = Some(name.clone());
                Ok(Value::Nil)
            }

            Expr::Export(symbols) => {
                // 現在のモジュール名を取得
                let module_name = self.current_module.clone()
                    .ok_or_else(|| "exportはmodule定義の中でのみ使用できます".to_string())?;

                // エクスポートする値を収集
                let mut exports = HashMap::new();
                for symbol in symbols {
                    if let Some(value) = env.borrow().get(symbol) {
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
                self.modules.insert(module_name, Rc::new(module));

                Ok(Value::Nil)
            }

            Expr::Use { module, mode } => {
                self.eval_use(module, mode, env)
            }

            Expr::Call { func, args } => {
                // 高階関数と論理演算子、quoteの特別処理
                if let Expr::Symbol(name) = func.as_ref() {
                    match name.as_str() {
                        "map" => return self.eval_map(args, env),
                        "filter" => return self.eval_filter(args, env),
                        "reduce" => return self.eval_reduce(args, env),
                        "and" => return self.eval_and(args, env),
                        "or" => return self.eval_or(args, env),
                        "quote" => return self.eval_quote(args),
                        _ => {}
                    }
                }

                let func_val = self.eval_with_env(func, env.clone())?;
                let arg_vals: Result<Vec<_>, _> = args
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                let arg_vals = arg_vals?;

                match func_val {
                    Value::NativeFunc(nf) => (nf.func)(&arg_vals),
                    Value::Function(f) => {
                        let parent_env = Rc::new(RefCell::new(f.env.clone()));
                        let mut new_env = Env::with_parent(parent_env);

                        if f.is_variadic {
                            // 可変長引数の処理
                            if f.params.len() != 1 {
                                return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["variadic fn", "1"]));
                            }
                            new_env.set(f.params[0].clone(), Value::List(arg_vals));
                        } else {
                            // 通常の引数
                            if f.params.len() != arg_vals.len() {
                                return Err(fmt_msg(
                                    MsgKey::ArgCountMismatch,
                                    &[&f.params.len().to_string(), &arg_vals.len().to_string()],
                                ));
                            }
                            for (param, arg) in f.params.iter().zip(arg_vals.iter()) {
                                new_env.set(param.clone(), arg.clone());
                            }
                        }

                        self.eval_with_env(&f.body, Rc::new(RefCell::new(new_env)))
                    }
                    _ => Err(fmt_msg(MsgKey::NotAFunction, &[&format!("{:?}", func_val)])),
                }
            }
        }
    }

    fn eval_match(
        &mut self,
        value: &Value,
        arms: &[MatchArm],
        env: Rc<RefCell<Env>>,
    ) -> Result<Value, String> {
        for arm in arms {
            let mut bindings = HashMap::new();
            if self.match_pattern(&arm.pattern, value, &mut bindings)? {
                // ガード条件のチェック
                if let Some(guard) = &arm.guard {
                    let mut guard_env = Env::with_parent(env.clone());
                    for (name, val) in &bindings {
                        guard_env.set(name.clone(), val.clone());
                    }
                    let guard_val = self.eval_with_env(guard, Rc::new(RefCell::new(guard_env)))?;
                    if !guard_val.is_truthy() {
                        continue;
                    }
                }

                // マッチ成功：バインディングを環境に追加して本体を評価
                let mut match_env = Env::with_parent(env.clone());
                for (name, val) in bindings {
                    match_env.set(name, val);
                }
                return self.eval_with_env(&arm.body, Rc::new(RefCell::new(match_env)));
            }
        }
        Err("どのパターンにもマッチしませんでした".to_string())
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
        }
    }

    /// map関数の実装: (map f coll)
    fn eval_map(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::map(&[func, coll], self)
    }

    /// filter関数の実装: (filter pred coll)
    fn eval_filter(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;
        builtins::filter(&[pred, coll], self)
    }

    /// reduce関数の実装: (reduce f init coll) または (reduce f coll)
    fn eval_reduce(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
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

    /// 関数を適用するヘルパー（builtinsモジュールから使用）
    pub fn apply_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, String> {
        self.apply_func(func, args.to_vec())
    }

    /// 関数を適用するヘルパー（内部用）
    fn apply_func(&mut self, func: &Value, args: Vec<Value>) -> Result<Value, String> {
        match func {
            Value::NativeFunc(nf) => (nf.func)(&args),
            Value::Function(f) => {
                let parent_env = Rc::new(RefCell::new(f.env.clone()));
                let mut new_env = Env::with_parent(parent_env);

                if f.is_variadic {
                    if f.params.len() != 1 {
                        return Err(msg(MsgKey::VariadicFnOneParam).to_string());
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

                self.eval_with_env(&f.body, Rc::new(RefCell::new(new_env)))
            }
            _ => Err(format!("関数ではありません: {:?}", func)),
        }
    }

    /// and論理演算子（短絡評価）
    fn eval_and(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
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
    fn eval_or(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
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

    /// ExprをValueに変換（評価せずに）
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
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::List(values?))
            }
            Expr::Vector(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::Vector(values?))
            }
            Expr::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match self.expr_to_value(k)? {
                        Value::Keyword(k) => k,
                        Value::String(s) => s,
                        Value::Symbol(s) => s,
                        _ => return Err("マップのキーは文字列またはキーワードが必要です".to_string()),
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
            // モジュール関連とtry、deferはquoteできない
            Expr::Module(_) | Expr::Export(_) | Expr::Use { .. } | Expr::Try(_) | Expr::Defer(_) => {
                Err("module/export/use/try/defer cannot be quoted".to_string())
            }
            _ => Err(format!("quoteできない式: {:?}", expr)),
        }
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
        &mut self,
        module_name: &str,
        mode: &crate::value::UseMode,
        env: Rc<RefCell<Env>>,
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
                        env.borrow_mut().set(name.clone(), value.clone());
                    } else {
                        return Err(fmt_msg(MsgKey::SymbolNotExported, &[name, module_name]));
                    }
                }
            }
            UseMode::All => {
                // 全てインポート
                for (name, value) in &module.exports {
                    env.borrow_mut().set(name.clone(), value.clone());
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
    fn load_module(&mut self, name: &str) -> Result<Rc<Module>, String> {
        // 既にロード済みならキャッシュから返す
        if let Some(module) = self.modules.get(name) {
            return Ok(module.clone());
        }

        // 循環参照チェック
        if self.loading_modules.contains(&name.to_string()) {
            return Err(format!(
                "循環参照を検出しました: {}",
                self.loading_modules.join(" -> ")
            ));
        }

        // ロード中のモジュールリストに追加
        self.loading_modules.push(name.to_string());

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
            .map_err(|e| format!("モジュール{}のパーサー初期化エラー: {}", name, e))?;

        let exprs = parser.parse_all()
            .map_err(|e| format!("モジュール{}のパースエラー: {}", name, e))?;

        // 新しい環境で評価
        let module_env = Rc::new(RefCell::new(Env::new()));

        // グローバル環境から組み込み関数をコピー
        let bindings: Vec<_> = self.global_env.borrow().bindings()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in bindings {
            module_env.borrow_mut().set(key, value);
        }

        // 式を順次評価
        for expr in exprs {
            self.eval_with_env(&expr, module_env.clone())?;
        }

        // ロード中リストから削除
        self.loading_modules.pop();

        // モジュールが登録されているか確認
        self.modules.get(name).cloned()
            .ok_or_else(|| format!("モジュール{}はexportを含む必要があります", name))
    }
}
