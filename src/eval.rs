use crate::builtins;
use crate::builtins::util::to_map_key;
use crate::error::QiError;
use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::lexer::Span;
use crate::value::{
    Env, Expr, FStringPart, Function, Macro, MatchArm, Module, NativeFunc, Pattern, Value,
};
use parking_lot::RwLock;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::sync::Arc;

// ========================================
// マジック文字列定数
// ========================================

/// Recur特殊エラーのプレフィックス
const RECUR_SENTINEL: &str = "__RECUR__:";

/// ドキュメント文字列のキープレフィックス
pub const DOC_PREFIX: &str = "__doc__";

/// 高階関数の内部状態キー
pub mod hof_keys {
    pub const COMPLEMENT_FUNC: &str = "__complement_func__";
    pub const JUXT_FUNCS: &str = "__juxt_funcs__";
    pub const TAP_FUNC: &str = "__tap_func__";
    pub const PARTIAL_FUNC: &str = "__partial_func__";
    pub const PARTIAL_ARGS: &str = "__partial_args__";
    pub const COMP_FUNCS: &str = "__comp_funcs__";
    pub const CONSTANTLY_VALUE: &str = "__constantly_value__";
    pub const PARTIAL_PLACEHOLDER: &str = "__partial_placeholder__";
}

// ========================================
// エラーヘルパー関数
// ========================================

/// QiErrorを使ってエラーメッセージを生成
#[inline]
fn qerr(key: MsgKey, args: &[&str]) -> String {
    let base_msg = fmt_msg(key, args);
    let error_code = QiError::error_code_from_eval_msg(&key);
    QiError::new(error_code, base_msg).into()
}

/// 環境から変数名の候補を取得
fn find_similar_names(env: &Env, target: &str, max_distance: usize, limit: usize) -> Vec<String> {
    // limit.max(8): 通常limit=3だが、フィルタ前の候補は8個程度と推定
    let mut candidates = Vec::with_capacity(limit.max(8));
    for (name, _) in env.bindings() {
        let distance = strsim::levenshtein(target, name);
        if distance <= max_distance {
            candidates.push((name.clone(), distance));
        }
    }

    // 距離でソート
    candidates.sort_by_key(|(_, dist)| *dist);

    // 上位のみ取得
    // limit: 通常3個まで取得
    let mut results = Vec::with_capacity(limit);
    for (name, _) in candidates.into_iter().take(limit) {
        results.push(name);
    }
    results
}

#[derive(Clone)]
pub struct Evaluator {
    global_env: Arc<RwLock<Env>>,
    defer_stack: Arc<RwLock<SmallVec<[Vec<Expr>; 4]>>>, // スコープごとのdeferスタック（LIFO、最大4層まで）
    modules: Arc<RwLock<HashMap<String, Arc<Module>>>>, // ロード済みモジュール
    current_module: Arc<RwLock<Option<String>>>,        // 現在評価中のモジュール名
    loading_modules: Arc<RwLock<Vec<String>>>,          // 循環参照検出用
    #[allow(dead_code)]
    call_stack: Arc<RwLock<Vec<String>>>, // 関数呼び出しスタック（スタックトレース用）
    source_name: Arc<RwLock<Option<String>>>,           // ソースファイル名または入力名
    source_code: Arc<RwLock<Option<String>>>,           // ソースコード全体
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

        // 特殊な関数を登録（print, list, vector, to-list, to-vector）
        env_rc.write().set(
            "print".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "print",
                func: native_print,
            }),
        );
        env_rc.write().set(
            "list".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "list",
                func: native_list,
            }),
        );
        env_rc.write().set(
            "vector".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "vector",
                func: native_vector,
            }),
        );
        env_rc.write().set(
            "to-list".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "to-list",
                func: native_to_list,
            }),
        );
        env_rc.write().set(
            "to-vector".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "to-vector",
                func: native_to_vector,
            }),
        );

        // 型判定関数（builtins以外のもの）
        env_rc.write().set(
            "number?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "number?",
                func: native_is_number,
            }),
        );
        env_rc.write().set(
            "fn?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "fn?",
                func: native_is_fn,
            }),
        );

        let evaluator = Evaluator {
            global_env: env_rc.clone(),
            defer_stack: Arc::new(RwLock::new(SmallVec::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
            current_module: Arc::new(RwLock::new(None)),
            loading_modules: Arc::new(RwLock::new(Vec::new())),
            call_stack: Arc::new(RwLock::new(Vec::new())),
            source_name: Arc::new(RwLock::new(None)),
            source_code: Arc::new(RwLock::new(None)),
        };

        // 標準マクロを定義
        evaluator.define_standard_macros();

        evaluator
    }

    /// 標準マクロを定義
    fn define_standard_macros(&self) {
        // tapは特別なEvaluator必要関数として別途登録済み
    }

    /// ソース名とソースコードを設定
    pub fn set_source(&self, name: String, code: String) {
        *self.source_name.write() = Some(name);
        *self.source_code.write() = Some(code);
    }

    /// Span情報を使ってエラーメッセージをフォーマット
    fn format_error_with_span(&self, message: String, span: &Span) -> String {
        // span.line, span.column が 0, 0 の場合は位置情報なし（dummy_span）
        if span.line == 0 && span.column == 0 {
            return message;
        }

        let source_name = self.source_name.read();
        let source_code = self.source_code.read();

        let mut result = message;

        // ファイル名と位置情報を追加
        if let Some(name) = source_name.as_ref() {
            result.push_str(&format!("\n  --> {}:{}:{}", name, span.line, span.column));
        } else {
            result.push_str(&format!(
                "\n  --> line {}, column {}",
                span.line, span.column
            ));
        }

        // ソースコードの該当行を表示
        if let Some(code) = source_code.as_ref() {
            let lines: Vec<&str> = code.lines().collect();
            if span.line > 0 && span.line <= lines.len() {
                let line_idx = span.line - 1;
                result.push_str(&format!("\n  |\n{:3} | {}", span.line, lines[line_idx]));

                // エラー位置にキャレット（^）を表示
                if span.column > 0 {
                    let spaces = " ".repeat(span.column - 1);
                    result.push_str(&format!("\n  | {spaces}^"));
                }
            }
        }

        result
    }

    pub fn eval(&self, expr: &Expr) -> Result<Value, String> {
        self.eval_with_env(expr, self.global_env.clone())
    }

    /// グローバル環境への参照を取得（REPL用）
    pub fn get_env(&self) -> Option<Arc<RwLock<Env>>> {
        Some(self.global_env.clone())
    }

    pub fn eval_with_env(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // ブレークポイントチェック (dap-server feature が有効な場合のみ)
        // リスト（関数呼び出し）の場合のみチェック
        #[cfg(feature = "dap-server")]
        {
            let span = expr.span();
            let should_check = matches!(expr, Expr::List { .. } | Expr::Call { .. });

            if should_check {
                let span = expr.span();
                let should_wait = {
                    let file_name = self
                        .source_name
                        .read()
                        .as_ref()
                        .unwrap_or(&"<input>".to_string())
                        .clone();

                    let mut guard = crate::debugger::GLOBAL_DEBUGGER.write();
                    if let Some(ref mut dbg) = *guard {
                        let result = dbg.check_breakpoint(
                            &file_name,
                            span.line,
                            span.column,
                            Some(env.clone()),
                        );
                        if result {
                            let log_msg =
                                format!("[EVAL] Breakpoint hit: {}:{}\n", file_name, span.line);
                            std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("/tmp/qi-dap.log")
                                .and_then(|mut f| {
                                    std::io::Write::write_all(&mut f, log_msg.as_bytes())
                                })
                                .ok();
                        }
                        result
                    } else {
                        let log_msg = "[EVAL] WARNING: GLOBAL_DEBUGGER is None\n";
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();
                        false
                    }
                };

                if should_wait {
                    let log_msg = "[EVAL] Waiting for debugger resume...\n";
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();
                    crate::debugger::wait_if_paused_global();
                    let log_msg = "[EVAL] Debugger resumed\n";
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();
                }
            }
        }

        match expr {
            Expr::Nil { .. } => Ok(Value::Nil),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::Integer { value, .. } => Ok(Value::Integer(*value)),
            Expr::Float { value, .. } => Ok(Value::Float(*value)),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::FString { parts, .. } => self.eval_fstring(parts, Arc::clone(&env)),
            Expr::Keyword { name, .. } => Ok(Value::Keyword(name.clone())),

            Expr::Symbol { name, span } => {
                let env_read = env.read();
                env_read.get(name).ok_or_else(|| {
                    // 類似した変数名を検索（最大編集距離3、最大3件）
                    let suggestions = find_similar_names(&env_read, name, 3, 3);
                    let msg = if suggestions.is_empty() {
                        qerr(MsgKey::UndefinedVar, &[name])
                    } else {
                        fmt_msg(
                            MsgKey::UndefinedVarWithSuggestions,
                            &[name, &suggestions.join(", ")],
                        )
                    };
                    self.format_error_with_span(msg, span)
                })
            }

            Expr::List { items, .. } => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval_with_env(item, Arc::clone(&env))?);
                }
                Ok(Value::List(values.into()))
            }

            Expr::Vector { items, .. } => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval_with_env(item, Arc::clone(&env))?);
                }
                Ok(Value::Vector(values.into()))
            }

            Expr::Map { pairs, .. } => {
                let mut map = HashMap::with_capacity(pairs.len());
                for (k, v) in pairs {
                    let key_value = self.eval_with_env(k, Arc::clone(&env))?;
                    let key = key_value.to_map_key()?;
                    let value = self.eval_with_env(v, Arc::clone(&env))?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map.into()))
            }

            Expr::Def {
                name,
                value,
                is_private,
                ..
            } => {
                // 名前衝突チェック（ただし__doc__で始まる変数は除外）
                if !name.starts_with(DOC_PREFIX) {
                    if let Some(existing) = env.read().get(name) {
                        match existing {
                            Value::NativeFunc(nf) => {
                                eprintln!("{}", qerr(MsgKey::RedefineBuiltin, &[name, nf.name]));
                            }
                            Value::Function(_) | Value::Macro(_) => {
                                eprintln!("{}", qerr(MsgKey::RedefineFunction, &[name]));
                            }
                            _ => {
                                eprintln!("{}", qerr(MsgKey::RedefineVariable, &[name]));
                            }
                        }
                    }
                }

                let val = self.eval_with_env(value, Arc::clone(&env))?;
                // 現在の環境に定義（プライベートフラグに応じて）
                if *is_private {
                    env.write().set_private(name.clone(), val.clone());
                } else {
                    env.write().set(name.clone(), val.clone());
                }
                Ok(val)
            }

            Expr::Fn {
                params,
                body,
                is_variadic,
                ..
            } => Ok(Value::Function(Arc::new(Function {
                params: params.clone(),
                body: (**body).clone(),
                env: Arc::clone(&env),
                is_variadic: *is_variadic,
                has_special_processing: false,
            }))),

            Expr::Let { bindings, body, .. } => {
                // let環境を一度だけArc<RwLock<Env>>として作成（cloneとヒープ確保を削減）
                let new_env = Arc::new(RwLock::new(Env::with_parent(Arc::clone(&env))));
                for (pattern, expr) in bindings {
                    let value = self.eval_with_env(expr, new_env.clone())?;
                    self.bind_fn_param(pattern, &value, &mut new_env.write())?;
                }
                self.eval_with_env(body, new_env)
            }

            Expr::If {
                test,
                then,
                otherwise,
                ..
            } => {
                let test_val = self.eval_with_env(test, Arc::clone(&env))?;
                if test_val.is_truthy() {
                    self.eval_with_env(then, env)
                } else if let Some(otherwise) = otherwise {
                    self.eval_with_env(otherwise, env)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::Do { exprs, .. } => self.eval_do(exprs, env),

            Expr::When {
                condition, body, ..
            } => {
                let cond_val = self.eval_with_env(condition, Arc::clone(&env))?;
                if cond_val.is_truthy() {
                    self.eval_do(body, env)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::While {
                condition, body, ..
            } => {
                loop {
                    let cond_val = self.eval_with_env(condition, Arc::clone(&env))?;
                    if !cond_val.is_truthy() {
                        break;
                    }
                    self.eval_do(body, Arc::clone(&env))?;
                }
                Ok(Value::Nil)
            }

            Expr::Until {
                condition, body, ..
            } => {
                loop {
                    let cond_val = self.eval_with_env(condition, Arc::clone(&env))?;
                    if cond_val.is_truthy() {
                        break;
                    }
                    self.eval_do(body, Arc::clone(&env))?;
                }
                Ok(Value::Nil)
            }

            Expr::WhileSome {
                binding,
                expr,
                body,
                ..
            } => {
                loop {
                    let val = self.eval_with_env(expr, Arc::clone(&env))?;
                    if matches!(val, Value::Nil) {
                        break;
                    }
                    {
                        env.write().set(binding.clone(), val);
                    }
                    self.eval_do(body, Arc::clone(&env))?;
                }
                Ok(Value::Nil)
            }

            Expr::UntilError {
                binding,
                expr,
                body,
                ..
            } => {
                loop {
                    let val = self.eval_with_env(expr, Arc::clone(&env))?;

                    // {:error ...} の場合は終了して値を返す
                    if let Value::Map(ref m) = val {
                        if m.contains_key(":error") {
                            return Ok(val);
                        }
                    }

                    {
                        env.write().set(binding.clone(), val);
                    }
                    self.eval_do(body, Arc::clone(&env))?;
                }
            }

            Expr::Match { expr, arms, .. } => {
                let value = self.eval_with_env(expr, Arc::clone(&env))?;
                self.eval_match(&value, arms, env)
            }

            Expr::Try { expr, .. } => self.eval_try(expr, env),

            Expr::Defer { expr, .. } => self.eval_defer(expr.as_ref()),

            Expr::Loop { bindings, body, .. } => self.eval_loop(bindings, body, env),

            Expr::Recur { args, .. } => self.eval_recur(args, env),

            Expr::Mac {
                name,
                params,
                is_variadic,
                body,
                ..
            } => self.eval_mac(name, params, *is_variadic, body.as_ref(), env),

            Expr::Quasiquote { expr, .. } => self.eval_quasiquote(expr, env, 0),

            Expr::Unquote { .. } => Err(msg(MsgKey::UnquoteOutsideQuasiquote).to_string()),

            Expr::UnquoteSplice { .. } => {
                Err(msg(MsgKey::UnquoteSpliceOutsideQuasiquote).to_string())
            }

            // モジュールシステム
            Expr::Module { name, .. } => {
                *self.current_module.write() = Some(name.clone());
                Ok(Value::Nil)
            }

            Expr::Export { symbols, .. } => {
                // 現在のモジュール名を取得
                let module_name = self
                    .current_module
                    .read()
                    .clone()
                    .ok_or_else(|| msg(MsgKey::ExportOnlyInModule).to_string())?;

                // シンボルの存在確認
                for symbol in symbols {
                    if env.read().get(symbol).is_none() {
                        return Err(qerr(MsgKey::SymbolNotFound, &[symbol, &module_name]));
                    }
                }

                // モジュールを登録または更新
                let module = Module {
                    name: module_name.clone(),
                    file_path: None,
                    env: Arc::clone(&env),
                    exports: Some(symbols.clone()),
                };
                self.modules.write().insert(module_name, Arc::new(module));

                Ok(Value::Nil)
            }

            Expr::Use { module, mode, .. } => self.eval_use(module, mode, env),

            Expr::Call { func, args, .. } => self.eval_call(func, args, env),
        }
    }

    /// 特殊形式（高階関数、演算子など）のディスパッチ
    ///
    /// 特殊な評価が必要な関数をチェックし、該当する場合は評価結果を返す。
    /// 該当しない場合はNoneを返す。
    #[inline]
    fn try_eval_special_form(
        &self,
        func: &Expr,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
    ) -> Option<Result<Value, String>> {
        if let Expr::Symbol { name, .. } = func {
            match name.as_str() {
                "_railway-pipe" => Some(self.eval_railway_pipe(args, env)),
                "and" => Some(self.eval_and(args, env)),
                "apply" => Some(self.eval_apply(args, env)),
                "go/catch" => Some(self.eval_catch(args, env)),
                "go/parallel-do" => Some(self.eval_parallel_do(args, env)),
                "go/pfilter" => Some(self.eval_pfilter(args, env)),
                "go/preduce" => Some(self.eval_preduce(args, env)),
                "go/run" => Some(self.eval_run(args, env)),
                "go/scope-go" => Some(self.eval_scope_go(args, env)),
                "go/select!" => Some(self.eval_select(args, env)),
                "go/then" => Some(self.eval_then(args, env)),
                "go/with-scope" => Some(self.eval_with_scope(args, env)),
                "branch" => Some(self.eval_branch(args, env)),
                "comp" => Some(self.eval_comp(args, env)),
                "drop-while" => Some(self.eval_drop_while(args, env)),
                "eval" => Some(self.eval_eval(args, env)),
                "list/every?" => Some(self.eval_every(args, env)),
                "filter" => Some(self.eval_filter(args, env)),
                "find" => Some(self.eval_find(args, env)),
                "list/chunk" => Some(self.eval_chunk(args, env)),
                "list/count-by" => Some(self.eval_count_by(args, env)),
                "list/drop-last" => Some(self.eval_drop_last(args, env)),
                "list/find-index" => Some(self.eval_find_index(args, env)),
                "list/group-by" => Some(self.eval_group_by(args, env)),
                "list/keep" => Some(self.eval_keep(args, env)),
                "list/max-by" => Some(self.eval_max_by(args, env)),
                "list/min-by" => Some(self.eval_min_by(args, env)),
                "list/partition-by" => Some(self.eval_partition_by(args, env)),
                "list/partition" => Some(self.eval_partition(args, env)),
                "list/sort-by" => Some(self.eval_sort_by(args, env)),
                "list/split-at" => Some(self.eval_split_at(args, env)),
                "list/sum-by" => Some(self.eval_sum_by(args, env)),
                "map-lines" => Some(self.eval_map_lines(args, env)),
                "map" => Some(self.eval_map(args, env)),
                "map/update-keys" => Some(self.eval_update_keys(args, env)),
                "map/update-vals" => Some(self.eval_update_vals(args, env)),
                "or" => Some(self.eval_or(args, env)),
                "go/pipeline-filter" => Some(self.eval_pipeline_filter(args, env)),
                "go/pipeline-map" => Some(self.eval_pipeline_map(args, env)),
                "go/pipeline" => Some(self.eval_pipeline(args, env)),
                "pmap" => Some(self.eval_pmap(args, env)),
                "each" => Some(self.eval_each(args, env)),
                "quote" => Some(self.eval_quote(args)),
                "reduce" => Some(self.eval_reduce(args, env)),
                "list/some?" => Some(self.eval_some(args, env)),
                "stream/filter" => Some(self.eval_stream_filter(args, env)),
                "stream/iterate" => Some(self.eval_iterate(args, env)),
                "stream/map" => Some(self.eval_stream_map(args, env)),
                "swap!" => Some(self.eval_swap(args, env)),
                "take-while" => Some(self.eval_take_while(args, env)),
                "tap" => Some(self.eval_tap(args, env)),
                "test/assert-throws" => Some(self.eval_test_assert_throws(args, env)),
                "test/run" => Some(self.eval_test_run(args, env)),
                "time" => Some(self.eval_time(args, env)),
                "update-in" => Some(self.eval_update_in(args, env)),
                "update" => Some(self.eval_update(args, env)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 関数呼び出しの評価
    ///
    /// 特別な関数（高階関数、論理演算子など）のディスパッチと、
    /// 通常の関数呼び出し（ネイティブ関数、ユーザー定義関数、マクロ、キーワード関数）を処理
    fn eval_call(
        &self,
        func: &Expr,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        // 特殊形式のチェック
        if let Some(result) = self.try_eval_special_form(func, args, Arc::clone(&env)) {
            return result;
        }

        let func_val = self.eval_with_env(func, Arc::clone(&env))?;

        // マクロの場合は展開してから評価
        if let Value::Macro(mac) = &func_val {
            let expanded = self.expand_macro(mac, args, Arc::clone(&env))?;
            return self.eval_with_env(&expanded, env);
        }

        let arg_vals: Result<SmallVec<[Value; 4]>, _> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
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
                    return Err(qerr(MsgKey::NeedExactlyNArgs, &["keyword", "1"]));
                }
                match &arg_vals[0] {
                    Value::Map(m) => m
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| qerr(MsgKey::KeyNotFound, &[&key])),
                    _ => Err(qerr(MsgKey::TypeOnly, &["keyword fn", "maps"])),
                }
            }
            _ => Err(fmt_msg(
                MsgKey::TypeMismatch,
                &["function", func_val.type_name(), &format!("{}", func_val)],
            )),
        }
    }

    fn eval_match(
        &self,
        value: &Value,
        arms: &[MatchArm],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        for arm in arms {
            let mut bindings: SmallVec<[(String, Value); 8]> = SmallVec::new();
            let mut transforms = Vec::new();

            if self.match_pattern_with_transforms(
                &arm.pattern,
                value,
                &mut bindings,
                &mut transforms,
            )? {
                // ガード条件のチェック
                if let Some(guard) = &arm.guard {
                    let mut guard_env = Env::with_parent(Arc::clone(&env));
                    for (name, val) in &bindings {
                        guard_env.set(name.clone(), val.clone());
                    }
                    let guard_val = self.eval_with_env(guard, Arc::new(RwLock::new(guard_env)))?;
                    if !guard_val.is_truthy() {
                        continue;
                    }
                }

                // 変換を適用
                let mut match_env = Env::with_parent(Arc::clone(&env));
                for (name, val) in bindings {
                    match_env.set(name.clone(), val.clone());
                }

                let match_env_rc = Arc::new(RwLock::new(match_env));

                // 変換を適用して環境を更新
                for (var, transform_expr, original_val) in transforms {
                    let result =
                        self.apply_transform(&transform_expr, &original_val, match_env_rc.clone())?;
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
        bindings: &mut SmallVec<[(String, Value); 8]>,
        transforms: &mut Vec<(String, Expr, Value)>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Transform(var, transform) => {
                // 変換情報を記録
                transforms.push((var.clone(), (**transform).clone(), value.clone()));
                bindings.push((var.clone(), value.clone()));
                Ok(true)
            }
            Pattern::Vector(patterns, rest) => {
                // ネストされたTransformパターンを扱うため、再帰的に処理
                let values = match value {
                    Value::Vector(v) | Value::List(v) => v,
                    _ => return Ok(false),
                };

                if patterns.len() > values.len() {
                    return Ok(false);
                }
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    if !self.match_pattern_with_transforms(pat, val, bindings, transforms)? {
                        return Ok(false);
                    }
                }

                // restパターンの処理
                if let Some(rest_pattern) = rest {
                    let rest_values: Vec<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern_with_transforms(
                        rest_pattern,
                        &Value::Vector(rest_values.into()),
                        bindings,
                        transforms,
                    )?;
                } else if patterns.len() != values.len() {
                    return Ok(false);
                }

                Ok(true)
            }
            Pattern::List(patterns, rest) => {
                // ネストされたTransformパターンを扱うため、再帰的に処理
                let values = match value {
                    Value::List(v) | Value::Vector(v) => v,
                    _ => return Ok(false),
                };

                if patterns.len() > values.len() {
                    return Ok(false);
                }
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    if !self.match_pattern_with_transforms(pat, val, bindings, transforms)? {
                        return Ok(false);
                    }
                }

                // restパターンの処理
                if let Some(rest_pattern) = rest {
                    let rest_values: Vec<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern_with_transforms(
                        rest_pattern,
                        &Value::List(rest_values.into()),
                        bindings,
                        transforms,
                    )?;
                } else if patterns.len() != values.len() {
                    return Ok(false);
                }

                Ok(true)
            }
            Pattern::Map(pattern_pairs, as_var) => {
                // ネストされたTransformパターンを扱うため、再帰的に処理
                if let Value::Map(map) = value {
                    for (key, pat) in pattern_pairs {
                        let map_key = to_map_key(key);
                        if let Some(val) = map.get(&map_key) {
                            if !self
                                .match_pattern_with_transforms(pat, val, bindings, transforms)?
                            {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    // :asパターンの処理
                    if let Some(var) = as_var {
                        bindings.push((var.clone(), value.clone()));
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::As(inner_pattern, var) => {
                // ネストされたTransformパターンを扱うため、再帰的に処理
                if self.match_pattern_with_transforms(inner_pattern, value, bindings, transforms)? {
                    bindings.push((var.clone(), value.clone()));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => {
                // Wildcard, Nil, Bool, Integer, Float, String, Keyword, Var, Or
                // これらはTransformを含まないので、通常のmatch_patternで処理
                self.match_pattern(pattern, value, bindings)
            }
        }
    }

    fn apply_transform(
        &self,
        transform: &Expr,
        value: &Value,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        // 変換式を評価して値に適用
        // transform が関数の場合: (transform value)
        // transform がシンボルの場合: (symbol value)
        let transform_val = self.eval_with_env(transform, Arc::clone(&env))?;
        self.apply_function(&transform_val, std::slice::from_ref(value))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut SmallVec<[(String, Value); 8]>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Nil => Ok(matches!(value, Value::Nil)),
            Pattern::Bool(b) => Ok(matches!(value, Value::Bool(vb) if vb == b)),
            Pattern::Integer(n) => Ok(matches!(value, Value::Integer(vn) if vn == n)),
            Pattern::Float(f) => {
                Ok(matches!(value, Value::Float(vf) if (vf - f).abs() < f64::EPSILON))
            }
            Pattern::String(s) => Ok(matches!(value, Value::String(vs) if vs == s)),
            Pattern::Keyword(k) => Ok(matches!(value, Value::Keyword(vk) if vk == k)),
            Pattern::Var(name) => {
                bindings.push((name.clone(), value.clone()));
                Ok(true)
            }
            Pattern::Vector(patterns, rest) => {
                // VectorパターンはVectorとListの両方にマッチ（一貫性のため）
                let values = match value {
                    Value::Vector(v) => v,
                    Value::List(v) => v,
                    _ => return Ok(false),
                };

                if patterns.len() > values.len() {
                    return Ok(false);
                }
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    if !self.match_pattern(pat, val, bindings)? {
                        return Ok(false);
                    }
                }

                // restパターンがある場合は残りの要素を束縛
                if let Some(rest_pattern) = rest {
                    let rest_values: Vec<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern(rest_pattern, &Value::Vector(rest_values.into()), bindings)?;
                } else if patterns.len() != values.len() {
                    // restパターンがない場合は要素数が一致しなければマッチ失敗
                    return Ok(false);
                }

                Ok(true)
            }
            Pattern::List(patterns, rest) => {
                // ListパターンはListとVectorの両方にマッチ
                let values = match value {
                    Value::List(v) => v,
                    Value::Vector(v) => v,
                    _ => return Ok(false),
                };

                if patterns.len() > values.len() {
                    return Ok(false);
                }
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    if !self.match_pattern(pat, val, bindings)? {
                        return Ok(false);
                    }
                }

                // restパターンがある場合は残りの要素を束縛
                if let Some(rest_pattern) = rest {
                    let rest_values: Vec<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern(rest_pattern, &Value::List(rest_values.into()), bindings)?;
                } else if patterns.len() != values.len() {
                    // restパターンがない場合は要素数が一致しなければマッチ失敗
                    return Ok(false);
                }

                Ok(true)
            }
            Pattern::Map(pattern_pairs, as_var) => {
                if let Value::Map(map) = value {
                    for (key, pat) in pattern_pairs {
                        // キーワードをマップキー形式に変換
                        let map_key = to_map_key(key);
                        if let Some(val) = map.get(&map_key) {
                            if !self.match_pattern(pat, val, bindings)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    // :asパターンの処理
                    if let Some(var) = as_var {
                        bindings.push((var.clone(), value.clone()));
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
                    bindings.push((var.clone(), value.clone()));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Or(patterns) => {
                // 各パターンを順番に試す（cloneせずにロールバック）
                for pat in patterns {
                    let start_len = bindings.len();
                    if self.match_pattern(pat, value, bindings)? {
                        // 最初にマッチしたパターンを使う
                        return Ok(true);
                    }
                    // 失敗時はロールバック
                    bindings.truncate(start_len);
                }
                // どれもマッチしなかった
                Ok(false)
            }
            Pattern::Transform(_, _) => {
                // Transformは match_pattern_with_transforms で処理される
                unreachable!("Transform pattern should be handled in match_pattern_with_transforms")
            }
        }
    }

    /// map関数の実装: (map f coll)
    fn eval_map(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::map(&[func, coll], self)
    }

    /// filter関数の実装: (filter pred coll)
    fn eval_filter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::filter(&[pred, coll], self)
    }

    /// reduce関数の実装: (reduce f init coll) または (reduce f coll)
    fn eval_reduce(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;

        if args.len() == 3 {
            let init = self.eval_with_env(&args[1], Arc::clone(&env))?;
            let coll = self.eval_with_env(&args[2], Arc::clone(&env))?;
            builtins::reduce(&[func, coll, init], self)
        } else {
            let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
            builtins::reduce(&[func, coll], self)
        }
    }

    /// swap!関数の実装: (swap! atom f args...)
    fn eval_swap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let atom = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let func = self.eval_with_env(&args[1], Arc::clone(&env))?;
        let mut swap_args = vec![atom, func];
        for arg in &args[2..] {
            swap_args.push(self.eval_with_env(arg, Arc::clone(&env))?);
        }
        builtins::swap(&swap_args, self)
    }

    /// eval関数の実装: (eval expr)
    fn eval_eval(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let expr = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::eval(&[expr], self)
    }

    /// pmap関数の実装: (pmap f coll)
    fn eval_pmap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::pmap(&[func, coll], self)
    }

    /// each関数の実装: (each func coll)
    fn eval_each(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::each(&[func, coll], self)
    }

    /// pfilter関数の実装: (pfilter pred coll)
    fn eval_pfilter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["go/pfilter"]));
        }
        let pred = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::pfilter(&[pred, coll], self)
    }

    /// preduce関数の実装: (preduce f coll init) - reduceと同じ順序
    fn eval_preduce(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 3 {
            return Err(qerr(MsgKey::NeedNArgsDesc, &["go/preduce", "3", ""]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        let init = self.eval_with_env(&args[2], Arc::clone(&env))?;
        builtins::preduce(&[func, coll, init], self)
    }

    fn eval_partition(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::partition(&[func, coll], self)
    }

    fn eval_group_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::group_by(&[func, coll], self)
    }

    fn eval_map_lines(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let text = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::map_lines(&[func, text], self)
    }

    fn eval_update(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let map = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let key = self.eval_with_env(&args[1], Arc::clone(&env))?;
        let func = self.eval_with_env(&args[2], Arc::clone(&env))?;
        builtins::update(&[map, key, func], self)
    }

    fn eval_update_in(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let map = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let path = self.eval_with_env(&args[1], Arc::clone(&env))?;
        let func = self.eval_with_env(&args[2], Arc::clone(&env))?;
        builtins::update_in(&[map, path, func], self)
    }

    fn eval_comp(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let funcs: Result<Vec<_>, _> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect();
        builtins::comp(&funcs?, self)
    }

    fn eval_apply(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let list = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::apply(&[func, list], self)
    }

    fn eval_take_while(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::take_while(&[pred, coll], self)
    }

    fn eval_drop_while(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let pred = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let coll = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::drop_while(&[pred, coll], self)
    }

    /// test/run - テストを実行して結果を記録
    fn eval_test_run(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["test/run"]));
        }
        let name = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let body = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::test_run(&[name, body], self)
    }

    /// test/assert-throws - 式が例外を投げることをアサート
    fn eval_test_assert_throws(
        &self,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["test/assert-throws"]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::test_assert_throws(&[func], self)
    }

    /// 関数を適用するヘルパー（builtinsモジュールから使用）
    pub fn apply_function(&self, func: &Value, args: &[Value]) -> Result<Value, String> {
        // NativeFuncの場合は直接呼び出し（SmallVec変換をスキップして高速化）
        if let Value::NativeFunc(nf) = func {
            return (nf.func)(args);
        }
        // ユーザー定義関数の場合のみSmallVec変換
        self.apply_func(func, args.iter().cloned().collect())
    }

    /// 関数名を取得（プロファイリング用）
    fn get_function_name(&self, func: &Value) -> Option<String> {
        if let Value::Function(f) = func {
            // グローバル環境から関数を検索
            for (name, value) in self.global_env.read().bindings() {
                if let Value::Function(stored_f) = value {
                    if Arc::ptr_eq(f, stored_f) {
                        return Some(name.clone());
                    }
                }
            }
        }
        None
    }

    /// Patternパターンを値にマッチさせて環境にバインド
    #[allow(clippy::only_used_in_recursion)]
    fn bind_fn_param(
        &self,
        param: &crate::value::Pattern,
        value: &Value,
        env: &mut Env,
    ) -> Result<(), String> {
        match param {
            Pattern::Var(name) => {
                env.set(name.clone(), value.clone());
                Ok(())
            }
            Pattern::List(params, rest_param) | Pattern::Vector(params, rest_param) => {
                // 値がリストまたはベクタであることを確認
                let values = match value {
                    Value::List(v) | Value::Vector(v) => v,
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeErrorVectorPattern,
                            &[value.type_name()]
                        ));
                    }
                };

                // restパターンがある場合とない場合で処理を分岐
                if let Some(rest) = rest_param {
                    // [x y ...rest] 形式
                    if values.len() < params.len() {
                        return Err(fmt_msg(
                            MsgKey::ArgErrorVectorPatternMinimum,
                            &[&params.len().to_string(), &values.len().to_string()]
                        ));
                    }

                    // 固定部分をバインド
                    for (p, v) in params.iter().zip(values.iter()) {
                        self.bind_fn_param(p, v, env)?;
                    }

                    // rest部分をバインド（残りの要素をリストとして）
                    let rest_values: im::Vector<Value> =
                        values.iter().skip(params.len()).cloned().collect();
                    self.bind_fn_param(rest, &Value::List(rest_values), env)?;
                } else {
                    // [x y] 形式（固定長）
                    if values.len() != params.len() {
                        return Err(fmt_msg(
                            MsgKey::ArgErrorVectorPattern,
                            &[&params.len().to_string(), &values.len().to_string()]
                        ));
                    }

                    for (p, v) in params.iter().zip(values.iter()) {
                        self.bind_fn_param(p, v, env)?;
                    }
                }
                Ok(())
            }
            Pattern::Map(pairs, as_var) => {
                // 値がマップであることを確認
                let map = match value {
                    Value::Map(m) => m,
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeErrorMapPattern,
                            &[value.type_name()]
                        ));
                    }
                };

                // 各キーに対応する値をバインド
                for (key, pattern) in pairs {
                    // キーワードをマップキー形式に変換
                    let map_key = to_map_key(key);
                    if let Some(val) = map.get(&map_key) {
                        self.bind_fn_param(pattern, val, env)?;
                    } else {
                        return Err(fmt_msg(MsgKey::KeyErrorMapMissing, &[key]));
                    }
                }

                // :as 変数があればマップ全体をバインド
                if let Some(var) = as_var {
                    env.set(var.clone(), value.clone());
                }

                Ok(())
            }
            Pattern::As(inner, var) => {
                // 内側のパターンをバインド
                self.bind_fn_param(inner, value, env)?;
                // 値全体も変数にバインド
                env.set(var.clone(), value.clone());
                Ok(())
            }
            // match専用パターン（fn/letでは使用不可）
            Pattern::Wildcard | Pattern::Nil | Pattern::Bool(_) | Pattern::Integer(_)
            | Pattern::Float(_) | Pattern::String(_) | Pattern::Keyword(_)
            | Pattern::Transform(_, _) | Pattern::Or(_) => {
                Err(fmt_msg(MsgKey::PatternErrorNotAllowed, &[]))
            }
        }
    }

    /// 関数を適用するヘルパー（内部用）
    fn apply_func(&self, func: &Value, args: SmallVec<[Value; 4]>) -> Result<Value, String> {
        match func {
            Value::NativeFunc(nf) => (nf.func)(&args),
            Value::Function(f) => {
                // 特殊処理フラグがtrueの場合のみ環境ルックアップ（99.9%の通常関数で高速化）
                if f.has_special_processing {
                    // complement特殊処理 - 実行前にチェック
                    if let Some(complement_func) = f.env.read().get(hof_keys::COMPLEMENT_FUNC) {
                        let result = self.apply_func(&complement_func, args)?;
                        return Ok(Value::Bool(!result.is_truthy()));
                    }

                    // juxt特殊処理 - 実行前にチェック
                    if let Some(Value::List(juxt_funcs)) = f.env.read().get(hof_keys::JUXT_FUNCS) {
                        let mut results = Vec::with_capacity(juxt_funcs.len());
                        for jfunc in &juxt_funcs {
                            let result = self.apply_func(jfunc, args.clone())?;
                            results.push(result);
                        }
                        return Ok(Value::Vector(results.into()));
                    }

                    // tap>特殊処理 - 副作用を実行してから値を返す
                    if let Some(tap_func) = f.env.read().get(hof_keys::TAP_FUNC) {
                        if args.len() == 1 {
                            let value = args[0].clone();
                            // 副作用関数を実行（結果は無視）
                            let _ = self.apply_func(&tap_func, smallvec![value.clone()]);
                            // 元の値をそのまま返す
                            return Ok(value);
                        }
                    }

                    // partial特殊処理 - 部分適用された引数と新しい引数を結合
                    if let Some(partial_func) = f.env.read().get(hof_keys::PARTIAL_FUNC) {
                        if let Some(Value::List(partial_args)) =
                            f.env.read().get(hof_keys::PARTIAL_ARGS)
                        {
                            // 部分適用された引数と新しい引数を結合
                            let mut combined_args: SmallVec<[Value; 4]> =
                                SmallVec::with_capacity(partial_args.len() + args.len());
                            combined_args.extend(partial_args.iter().cloned());
                            combined_args.extend(args.iter().cloned());
                            return self.apply_func(&partial_func, combined_args);
                        }
                    }

                    // comp特殊処理 - 関数合成（右から左に適用）
                    if let Some(Value::List(comp_funcs)) = f.env.read().get(hof_keys::COMP_FUNCS) {
                        if args.len() != 1 {
                            return Err(fmt_msg(
                                MsgKey::ArgCountMismatch,
                                &["1", &args.len().to_string()],
                            ));
                        }
                        let mut result = args[0].clone();
                        // 右から左に順番に適用
                        for func in comp_funcs.iter().rev() {
                            result = self.apply_func(func, smallvec![result])?;
                        }
                        return Ok(result);
                    }
                }

                // 通常の関数処理
                let parent_env = Arc::clone(&f.env);
                let mut new_env = Env::with_parent(parent_env);

                if f.is_variadic {
                    // 可変長引数関数は最低1つのパラメータが必要（可変長引数自体）
                    if f.params.is_empty() {
                        return Err(
                            "内部エラー: 可変長引数関数にパラメータがありません".to_string()
                        );
                    }

                    // 固定引数の数（可変長引数を除く）
                    let fixed_param_count = f.params.len() - 1;

                    // 引数の数が固定引数の数より少ない場合はエラー
                    if args.len() < fixed_param_count {
                        return Err(fmt_msg(
                            MsgKey::ArgCountMismatch,
                            &[
                                &format!("{}以上", fixed_param_count),
                                &args.len().to_string(),
                            ],
                        ));
                    }

                    // 固定引数をバインド
                    for (param, arg) in f.params.iter().take(fixed_param_count).zip(args.iter()) {
                        self.bind_fn_param(param, arg, &mut new_env)?;
                    }

                    // 残りの引数を可変長引数にバインド
                    let variadic_param = &f.params[fixed_param_count];
                    let remaining_args: Vec<Value> =
                        args.iter().skip(fixed_param_count).cloned().collect();

                    if let crate::value::Pattern::Var(name) = variadic_param {
                        new_env.set(
                            name.clone(),
                            Value::List(remaining_args.into_iter().collect()),
                        );
                    } else {
                        return Err(
                            "内部エラー: variadic引数がSimpleパターンではありません".to_string()
                        );
                    }
                } else {
                    if f.params.len() != args.len() {
                        return Err(fmt_msg(
                            MsgKey::ArgCountMismatch,
                            &[&f.params.len().to_string(), &args.len().to_string()],
                        ));
                    }
                    for (param, arg) in f.params.iter().zip(args.iter()) {
                        self.bind_fn_param(param, arg, &mut new_env)?;
                    }
                }

                // デバッガが有効な場合、関数呼び出しを記録
                #[cfg(feature = "dap-server")]
                {
                    let debugger_enabled = crate::debugger::GLOBAL_DEBUGGER.read().is_some();
                    if debugger_enabled {
                        // 関数名を取得
                        let func_name = self
                            .get_function_name(func)
                            .unwrap_or_else(|| "<anonymous>".to_string());

                        // ファイル名を取得
                        let file_name = self
                            .source_name
                            .read()
                            .as_ref()
                            .unwrap_or(&"<input>".to_string())
                            .clone();

                        // 関数本体のspan情報を取得
                        let span = f.body.span();

                        // デバッガに関数呼び出しを通知
                        let should_wait = {
                            let mut guard = crate::debugger::GLOBAL_DEBUGGER.write();
                            if let Some(ref mut dbg) = *guard {
                                dbg.enter_function(&func_name, &file_name, span.line, span.column);

                                // ブレークポイントチェック（new_envをArcでラップして渡す）
                                dbg.check_breakpoint(
                                    &file_name,
                                    span.line,
                                    span.column,
                                    Some(Arc::new(RwLock::new(new_env.clone()))),
                                )
                            } else {
                                false
                            }
                        };

                        // ロックを解放してから待機
                        if should_wait {
                            crate::debugger::wait_if_paused_global();
                        }

                        // 関数本体を実行
                        let result = if builtins::profile::is_enabled() {
                            let start = std::time::Instant::now();
                            let r = self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)));
                            let duration = start.elapsed();
                            builtins::profile::record_call(&func_name, duration);
                            r
                        } else {
                            self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)))
                        };

                        // デバッガに関数終了を通知
                        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
                            dbg.exit_function();
                        }

                        result
                    } else {
                        // デバッガ無効時（プロファイリングのみ）
                        if builtins::profile::is_enabled() {
                            let start = std::time::Instant::now();
                            let result =
                                self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)));
                            let duration = start.elapsed();

                            // 関数名を取得（環境から逆引き）
                            let func_name = self
                                .get_function_name(func)
                                .unwrap_or_else(|| "<anonymous>".to_string());
                            builtins::profile::record_call(&func_name, duration);

                            result
                        } else {
                            self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)))
                        }
                    }
                }
                #[cfg(not(feature = "dap-server"))]
                {
                    // デバッガ無効時（プロファイリングのみ）
                    if builtins::profile::is_enabled() {
                        let start = std::time::Instant::now();
                        let result = self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)));
                        let duration = start.elapsed();

                        // 関数名を取得（環境から逆引き）
                        let func_name = self
                            .get_function_name(func)
                            .unwrap_or_else(|| "<anonymous>".to_string());
                        builtins::profile::record_call(&func_name, duration);

                        result
                    } else {
                        self.eval_with_env(&f.body, Arc::new(RwLock::new(new_env)))
                    }
                }
            }
            _ => Err(fmt_msg(
                MsgKey::TypeMismatch,
                &["function", func.type_name(), &format!("{}", func)],
            )),
        }
    }

    /// and論理演算子（短絡評価）
    fn eval_and(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Bool(true));
        }
        let mut last = Value::Bool(true);
        for arg in args {
            last = self.eval_with_env(arg, Arc::clone(&env))?;
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
            let val = self.eval_with_env(arg, Arc::clone(&env))?;
            if val.is_truthy() {
                return Ok(val);
            }
        }
        Ok(Value::Nil)
    }

    /// quote - 式を評価せずにそのまま返す
    fn eval_quote(&self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["quote"]));
        }
        self.expr_to_value(&args[0])
    }

    /// 2引数のbuiltin関数を評価する共通ヘルパー
    #[inline]
    fn eval_builtin_2_args(
        &self,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
        func_name: &str,
        builtin: fn(&[Value], &Evaluator) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &[func_name]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtin(&vals, self)
    }

    /// 3引数のbuiltin関数を評価する共通ヘルパー
    #[inline]
    fn eval_builtin_3_args(
        &self,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
        func_name: &str,
        builtin: fn(&[Value], &Evaluator) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if args.len() != 3 {
            return Err(qerr(MsgKey::NeedNArgsDesc, &[func_name, "3", ""]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtin(&vals, self)
    }

    /// sort-by - キー関数でソート
    fn eval_sort_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "sort-by", builtins::sort_by)
    }

    /// chunk - 固定サイズでリストを分割
    fn eval_chunk(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["chunk"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::list::native_chunk(&vals)
    }

    /// count-by - 述語でカウント
    fn eval_count_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "count-by", builtins::count_by)
    }

    /// max-by - キー関数で最大値を取得
    fn eval_max_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "max-by", builtins::max_by)
    }

    /// min-by - キー関数で最小値を取得
    fn eval_min_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "min-by", builtins::min_by)
    }

    /// sum-by - キー関数で合計
    fn eval_sum_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "sum-by", builtins::sum_by)
    }

    fn eval_pipeline(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_3_args(args, env, "pipeline", builtins::pipeline)
    }

    fn eval_pipeline_map(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_3_args(args, env, "pipeline-map", builtins::pipeline_map)
    }

    fn eval_pipeline_filter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_3_args(args, env, "pipeline-filter", builtins::pipeline_filter)
    }

    fn eval_railway_pipe(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        self.eval_builtin_2_args(args, env, "_railway-pipe", builtins::railway_pipe)
    }

    fn eval_time(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Err(qerr(MsgKey::Need1Arg, &["time"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::time(&vals, self)
    }

    fn eval_tap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["tap"]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let value = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::tap(&[func, value], self)
    }

    fn eval_branch(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::branch(&vals, self)
    }

    fn eval_then(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["go/then"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::then(&vals, self)
    }

    fn eval_catch(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["go/catch"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::catch(&vals, self)
    }

    fn eval_select(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["go/select!"]));
        }
        let val = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::select(&[val], self)
    }

    fn eval_scope_go(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["go/scope-go"]));
        }
        let scope = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let func = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::scope_go(&[scope, func], self)
    }

    fn eval_with_scope(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["go/with-scope"]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::with_scope(&[func], self)
    }

    fn eval_run(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["go/run"]));
        }
        let val = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::run(&[val], self)
    }

    fn eval_parallel_do(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Vector(vec![].into()));
        }

        // 各式を遅延評価のために関数でラップ
        let funcs: Vec<Value> = args
            .iter()
            .map(|expr| {
                // 0引数の関数として作成: (fn [] expr)
                Value::Function(Arc::new(crate::value::Function {
                    params: vec![],
                    body: expr.clone(),
                    env: Arc::clone(&env),
                    is_variadic: false,
                    has_special_processing: false,
                }))
            })
            .collect();

        builtins::parallel_do(&funcs, self)
    }

    fn eval_iterate(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["iterate"]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let init = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::iterate(&[func, init], self)
    }

    fn eval_stream_map(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["stream-map"]));
        }
        let func = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let stream = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::stream_map(&[func, stream], self)
    }

    fn eval_stream_filter(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["stream-filter"]));
        }
        let pred = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let stream = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::stream_filter(&[pred, stream], self)
    }

    fn eval_find(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["find"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::find(&vals, self)
    }

    fn eval_find_index(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["find-index"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::find_index(&vals, self)
    }

    fn eval_every(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["list/every?"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::every(&vals, self)
    }

    fn eval_some(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["list/some?"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::some(&vals, self)
    }

    fn eval_update_keys(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["update-keys"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::update_keys(&vals, self)
    }

    fn eval_update_vals(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["update-vals"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::update_vals(&vals, self)
    }

    fn eval_partition_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["partition-by"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::partition_by(&vals, self)
    }

    fn eval_keep(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["keep"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::keep(&vals, self)
    }

    fn eval_drop_last(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["drop-last"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::list::native_drop_last(&vals)
    }

    fn eval_split_at(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["split-at"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::list::native_split_at(&vals)
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
    Ok(Value::List(args.iter().cloned().collect()))
}

/// vector - ベクタを作成
fn native_vector(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Vector(args.iter().cloned().collect()))
}

/// to-list - List/VectorをListに変換
fn native_to_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(qerr(MsgKey::Need1Arg, &["to-list"]));
    }
    match &args[0] {
        Value::List(_) => Ok(args[0].clone()),
        Value::Vector(v) => Ok(Value::List(v.clone())),
        _ => Err(qerr(MsgKey::TypeOnly, &["to-list", "lists or vectors"])),
    }
}

/// to-vector - List/VectorをVectorに変換
fn native_to_vector(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(qerr(MsgKey::Need1Arg, &["to-vector"]));
    }
    match &args[0] {
        Value::Vector(_) => Ok(args[0].clone()),
        Value::List(v) => Ok(Value::Vector(v.clone())),
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["to-vector", "lists or vectors"],
        )),
    }
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
    Ok(Value::Bool(matches!(
        args[0],
        Value::Integer(_) | Value::Float(_)
    )))
}

/// fn? - 関数かどうか判定
fn native_is_fn(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "fn?");
    Ok(Value::Bool(matches!(
        args[0],
        Value::Function(_) | Value::NativeFunc(_)
    )))
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
                    if module.is_exported(name) {
                        if let Some(value) = module.env.read().get(name) {
                            env.write().set(name.clone(), value);
                        } else {
                            return Err(qerr(MsgKey::SymbolNotFound, &[name, module_name]));
                        }
                    } else {
                        return Err(qerr(MsgKey::SymbolNotExported, &[name, module_name]));
                    }
                }
            }
            UseMode::All => {
                // 全ての公開シンボルをインポート（デッドロック回避のため先に収集）
                let bindings: Vec<(String, Value)> = {
                    let env_guard = module.env.read();
                    let all_bindings: Vec<_> = env_guard
                        .all_bindings()
                        .map(|(name, binding)| (name.clone(), binding.clone()))
                        .collect();
                    std::mem::drop(env_guard); // 明示的にロックを解放

                    // exportリストに基づいてフィルタ
                    all_bindings
                        .into_iter()
                        .filter(|(name, binding)| {
                            match &module.exports {
                                None => !binding.is_private,       // exportなし = privateでなければ公開
                                Some(list) => list.contains(name), // exportあり = リストに含まれていれば公開
                            }
                        })
                        .map(|(name, binding)| (name, binding.value))
                        .collect()
                };

                for (name, value) in bindings {
                    env.write().set(name, value);
                }
            }
            UseMode::As(alias) => {
                // エイリアス機能: alias/name という形式で全ての公開関数をインポート（デッドロック回避のため先に収集）
                let bindings: Vec<(String, Value)> = {
                    let env_guard = module.env.read();
                    let all_bindings: Vec<_> = env_guard
                        .all_bindings()
                        .map(|(name, binding)| (name.clone(), binding.clone()))
                        .collect();
                    std::mem::drop(env_guard); // 明示的にロックを解放

                    // exportリストに基づいてフィルタ
                    all_bindings
                        .into_iter()
                        .filter(|(name, binding)| {
                            match &module.exports {
                                None => !binding.is_private,       // exportなし = privateでなければ公開
                                Some(list) => list.contains(name), // exportあり = リストに含まれていれば公開
                            }
                        })
                        .map(|(name, binding)| (name, binding.value))
                        .collect()
                };

                for (name, value) in bindings {
                    let aliased_name = format!("{}/{}", alias, name);
                    env.write().set(aliased_name, value);
                }
            }
        }

        Ok(Value::Nil)
    }

    /// パッケージ検索パスを解決
    fn resolve_module_path(&self, name: &str) -> Result<Vec<String>, String> {
        let mut paths = Vec::new();

        // 絶対パスまたは相対パスの場合はそのまま使用
        if name.starts_with("./") || name.starts_with("../") || name.starts_with("/") {
            paths.push(format!("{}.qi", name));
            return Ok(paths);
        }

        // 1. プロジェクトローカル: ./qi_packages/{name}/mod.qi
        paths.push(format!("./qi_packages/{}/mod.qi", name));

        // 2. グローバルキャッシュ: ~/.qi/packages/{name}/{version}/mod.qi
        #[cfg(feature = "repl")]
        {
            if let Some(home) = dirs::home_dir() {
                let packages_dir = home.join(".qi").join("packages").join(name);

                // バージョンディレクトリを探す（最新版を使用）
                if let Ok(entries) = std::fs::read_dir(&packages_dir) {
                    let mut versions: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect();

                    // セマンティックバージョニングでソート（簡易版）
                    versions.sort_by(|a, b| {
                        let a_parts: Vec<u32> =
                            a.split('.').filter_map(|s| s.parse().ok()).collect();
                        let b_parts: Vec<u32> =
                            b.split('.').filter_map(|s| s.parse().ok()).collect();
                        b_parts.cmp(&a_parts) // 降順（新しい順）
                    });

                    // 最新バージョンのmod.qiを追加
                    if let Some(latest) = versions.first() {
                        paths.push(
                            packages_dir
                                .join(latest)
                                .join("mod.qi")
                                .to_string_lossy()
                                .to_string(),
                        );
                    }
                }
            }
        }

        // 3. 後方互換性: カレントディレクトリの.qiファイル
        paths.push(format!("{}.qi", name));

        // 4. 後方互換性: examples/
        paths.push(format!("examples/{}.qi", name));

        Ok(paths)
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
                return Err(fmt_msg(
                    MsgKey::CircularDependency,
                    &[&loading.join(" -> ")],
                ));
            }
        }

        // ロード中のモジュールリストに追加
        self.loading_modules.write().push(name.to_string());

        // パッケージ検索パスを解決
        let paths = self.resolve_module_path(name)?;

        let mut content = None;
        let mut found_path = None;
        for path in &paths {
            if let Ok(c) = std::fs::read_to_string(path) {
                content = Some(c);
                found_path = Some(path.clone());
                break;
            }
        }

        let content = content.ok_or_else(|| qerr(MsgKey::ModuleNotFound, &[name]))?;

        // デバッグ: ロードしたパスを表示（開発時のみ）
        if std::env::var("QI_DEBUG").is_ok() {
            eprintln!(
                "[DEBUG] Loaded module '{}' from: {}",
                name,
                found_path.as_deref().unwrap_or_default()
            );
        }

        // パースして評価
        let mut parser = crate::parser::Parser::new(&content)
            .map_err(|e| qerr(MsgKey::ModuleParserInitError, &[name, &e]))?;

        let exprs = parser
            .parse_all()
            .map_err(|e| qerr(MsgKey::ModuleParseError, &[name, &e]))?;

        // 新しい環境で評価
        let module_env = Arc::new(RwLock::new(Env::new()));

        // グローバル環境から組み込み関数をコピー
        let bindings: Vec<_> = self
            .global_env
            .read()
            .bindings()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in bindings {
            module_env.write().set(key, value);
        }

        // 現在のモジュール名をクリア（評価前の状態に戻す）
        let prev_module = self.current_module.read().clone();

        // 式を順次評価
        for expr in exprs {
            self.eval_with_env(&expr, module_env.clone())?;
        }

        // ロード中リストから削除
        self.loading_modules.write().pop();

        // モジュールが登録されているか確認、なければデフォルトで全公開モジュールを作成
        let module = {
            let modules_guard = self.modules.read();
            let existing = modules_guard.get(name).cloned();
            std::mem::drop(modules_guard); // 明示的にロックを解放

            if let Some(m) = existing {
                m
            } else {
                // exportがない場合は全公開モジュールとして登録
                let module_name = self.current_module.read().clone().unwrap_or_else(|| {
                    // モジュール名が設定されていない場合はファイル名から取得
                    std::path::Path::new(name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(name)
                        .to_string()
                });

                let module = Arc::new(Module {
                    name: module_name.clone(),
                    file_path: found_path,
                    env: module_env.clone(),
                    exports: None, // None = 全公開（defn-以外）
                });

                self.modules
                    .write()
                    .insert(name.to_string(), module.clone());
                module
            }
        };

        // 現在のモジュール名を元に戻す
        *self.current_module.write() = prev_module;

        Ok(module)
    }

    /// f-stringを評価
    fn eval_fstring(&self, parts: &[FStringPart], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let mut result = String::new();

        for part in parts {
            match part {
                FStringPart::Text(text) => result.push_str(text),
                FStringPart::Code(code) => {
                    // コードをパースして評価
                    let mut parser = crate::parser::Parser::new(code).map_err(|e| {
                        crate::i18n::fmt_msg(crate::i18n::MsgKey::FStringCodeParseError, &[&e])
                    })?;
                    let expr = parser.parse().map_err(|e| {
                        crate::i18n::fmt_msg(crate::i18n::MsgKey::FStringCodeParseError, &[&e])
                    })?;
                    let value = self.eval_with_env(&expr, Arc::clone(&env))?;

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
                            let strs: Vec<_> = items.iter().map(|v| format!("{}", v)).collect();
                            format!("({})", strs.join(" "))
                        }
                        Value::Vector(items) => {
                            let strs: Vec<_> = items.iter().map(|v| format!("{}", v)).collect();
                            format!("[{}]", strs.join(" "))
                        }
                        Value::Map(m) => {
                            let strs: Vec<_> =
                                m.iter().map(|(k, v)| format!(":{} {}", k, v)).collect();
                            format!("{{{}}}", strs.join(" "))
                        }
                        Value::Function(_) => "<function>".to_string(),
                        Value::NativeFunc(nf) => format!("<native-fn:{}>", nf.name),
                        Value::Macro(m) => format!("<macro:{}>", m.name),
                        Value::Atom(a) => format!("<atom:{}>", a.read()),
                        Value::Channel(_) => "<channel>".to_string(),
                        Value::Scope(_) => "<scope>".to_string(),
                        Value::Stream(_) => "<stream>".to_string(),
                        Value::Uvar(id) => format!("<uvar:{}>", id),
                    };
                    result.push_str(&s);
                }
            }
        }

        Ok(Value::String(result))
    }

    /// doを評価
    fn eval_do(&self, exprs: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // deferスコープを作成
        self.defer_stack.write().push(Vec::new());

        let mut result = Value::Nil;
        for expr in exprs {
            result = self.eval_with_env(expr, Arc::clone(&env))?;
        }

        // deferを実行（LIFO順）
        let defers = self.defer_stack.write().pop();
        if let Some(defers) = defers {
            for defer_expr in defers.iter().rev() {
                let _ = self.eval_with_env(defer_expr, Arc::clone(&env));
            }
        }

        Ok(result)
    }

    /// tryを評価
    ///
    /// **新仕様: {:error}以外は全て成功**
    /// - 成功 → 値そのまま（:okラップなし！）
    /// - エラー → {:error message}
    ///
    /// # 例
    /// ```ignore
    /// (try (+ 1 2))  ;; => 3
    /// (try (/ 1 0))  ;; => {:error "ゼロ除算エラー"}
    /// ```
    fn eval_try(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // Tryもdeferスコープを作成
        self.defer_stack.write().push(Vec::new());

        let result = match self.eval_with_env(expr, Arc::clone(&env)) {
            Ok(value) => Ok(value), // :okラップなし！
            Err(e) => Ok(Value::error(e)),
        };

        // deferを実行（LIFO順、エラーでも必ず実行）
        let defers = self.defer_stack.write().pop();
        if let Some(defers) = defers {
            for defer_expr in defers.iter().rev() {
                let _ = self.eval_with_env(defer_expr, Arc::clone(&env));
            }
        }

        result
    }

    /// deferを評価
    fn eval_defer(&self, expr: &Expr) -> Result<Value, String> {
        // defer式をスタックに追加（評価はしない）
        let mut stack = self.defer_stack.write();
        if let Some(current_scope) = stack.last_mut() {
            current_scope.push(expr.clone());
        } else {
            stack.push(vec![expr.clone()]);
        }
        Ok(Value::Nil)
    }

    /// recurを評価
    fn eval_recur(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // 引数を評価
        let values: Result<Vec<_>, _> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect();
        let values = values?;

        // Recurは特別なエラーとして扱う
        Err(format!("{}{}", RECUR_SENTINEL, values.len()))
    }

    /// macroを評価
    fn eval_mac(
        &self,
        name: &str,
        params: &[String],
        is_variadic: bool,
        body: &Expr,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        let mac = Macro {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.clone(),
            env: Arc::clone(&env),
            is_variadic,
        };
        env.write()
            .set(name.to_string(), Value::Macro(Arc::new(mac)));
        Ok(Value::Symbol(name.to_string()))
    }

    /// loopを評価
    fn eval_loop(
        &self,
        bindings: &[(String, Expr)],
        body: &Expr,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        // ループ用の環境を作成
        let mut loop_env = Env::with_parent(Arc::clone(&env));

        // 初期値で環境を設定
        let mut current_values = Vec::with_capacity(bindings.len());
        for (_name, expr) in bindings {
            let value = self.eval_with_env(expr, Arc::clone(&env))?;
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
                Err(e) if e.starts_with(RECUR_SENTINEL) => {
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
                            return Err(fmt_msg(
                                MsgKey::RecurArgCountMismatch,
                                &[&bindings.len().to_string(), &new_values.len().to_string()],
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
            Expr::Recur { args, .. } => Some(args),
            Expr::If {
                then, otherwise, ..
            } => Self::find_recur(then)
                .or_else(|| otherwise.as_ref().and_then(|e| Self::find_recur(e))),
            Expr::Do { exprs, .. } => exprs.iter().find_map(Self::find_recur),
            _ => None,
        }
    }

    /// quasiquoteを評価
    fn eval_quasiquote(
        &self,
        expr: &Expr,
        env: Arc<RwLock<Env>>,
        depth: usize,
    ) -> Result<Value, String> {
        match expr {
            Expr::Unquote { expr: e, .. } if depth == 0 => {
                // depth 0のunquoteは評価
                self.eval_with_env(e, env)
            }
            Expr::Unquote { expr: e, .. } => {
                // ネストしたquasiquote内のunquote
                let inner = self.eval_quasiquote(e, env, depth - 1)?;
                Ok(inner)
            }
            Expr::Quasiquote { expr: e, .. } => {
                // ネストしたquasiquote
                self.eval_quasiquote(e, env, depth + 1)
            }
            Expr::List { items, .. } => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    if let Expr::UnquoteSplice { expr: e, .. } = item {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(e, Arc::clone(&env))?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => {
                                    return Err(
                                        msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()
                                    )
                                }
                            }
                        } else {
                            let val = self.eval_quasiquote(e, Arc::clone(&env), depth - 1)?;
                            result.push(val);
                        }
                    } else {
                        let val = self.eval_quasiquote(item, Arc::clone(&env), depth)?;
                        result.push(val);
                    }
                }
                Ok(Value::List(result.into()))
            }
            Expr::Vector { items, .. } => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    if let Expr::UnquoteSplice { expr: e, .. } = item {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(e, Arc::clone(&env))?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => {
                                    return Err(
                                        msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()
                                    )
                                }
                            }
                        } else {
                            let val = self.eval_quasiquote(e, Arc::clone(&env), depth - 1)?;
                            result.push(val);
                        }
                    } else {
                        let val = self.eval_quasiquote(item, Arc::clone(&env), depth)?;
                        result.push(val);
                    }
                }
                Ok(Value::Vector(result.into()))
            }
            Expr::Call { func, args, .. } => {
                // Callもリストとして扱う
                let mut result = vec![self.eval_quasiquote(func, Arc::clone(&env), depth)?];
                for arg in args {
                    if let Expr::UnquoteSplice { expr: e, .. } = arg {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(e, Arc::clone(&env))?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => {
                                    return Err(
                                        msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()
                                    )
                                }
                            }
                        } else {
                            let val = self.eval_quasiquote(e, Arc::clone(&env), depth - 1)?;
                            result.push(val);
                        }
                    } else {
                        let val = self.eval_quasiquote(arg, Arc::clone(&env), depth)?;
                        result.push(val);
                    }
                }
                Ok(Value::List(result.into()))
            }
            // 特殊形式もリスト形式に変換
            Expr::If {
                test,
                then,
                otherwise,
                ..
            } => {
                let mut result = vec![Value::Symbol("if".to_string())];
                result.push(self.eval_quasiquote(test, Arc::clone(&env), depth)?);
                result.push(self.eval_quasiquote(then, Arc::clone(&env), depth)?);
                if let Some(o) = otherwise {
                    result.push(self.eval_quasiquote(o, Arc::clone(&env), depth)?);
                }
                Ok(Value::List(result.into()))
            }
            Expr::Do { exprs, .. } => {
                let mut result = vec![Value::Symbol("do".to_string())];
                for e in exprs {
                    if let Expr::UnquoteSplice { expr: us, .. } = e {
                        if depth == 0 {
                            // unquote-spliceは評価してリストを展開
                            let val = self.eval_with_env(us, Arc::clone(&env))?;
                            match val {
                                Value::List(v) | Value::Vector(v) => {
                                    result.extend(v);
                                }
                                _ => {
                                    return Err(
                                        msg(MsgKey::UnquoteSpliceNeedsListOrVector).to_string()
                                    )
                                }
                            }
                        } else {
                            result.push(self.eval_quasiquote(us, Arc::clone(&env), depth - 1)?);
                        }
                    } else {
                        result.push(self.eval_quasiquote(e, Arc::clone(&env), depth)?);
                    }
                }
                Ok(Value::List(result.into()))
            }
            Expr::Fn {
                params,
                body,
                is_variadic,
                ..
            } => {
                let mut items = vec![Value::Symbol("fn".to_string())];
                let param_vals: Vec<Value> = if *is_variadic && params.len() == 1 {
                    vec![
                        Value::Symbol("&".to_string()),
                        self.fn_param_to_value(&params[0]),
                    ]
                } else if *is_variadic {
                    let mut v: Vec<Value> = params[..params.len() - 1]
                        .iter()
                        .map(|p| self.fn_param_to_value(p))
                        .collect();
                    v.push(Value::Symbol("&".to_string()));
                    v.push(self.fn_param_to_value(&params[params.len() - 1]));
                    v
                } else {
                    params.iter().map(|p| self.fn_param_to_value(p)).collect()
                };
                items.push(Value::Vector(param_vals.into()));
                items.push(self.eval_quasiquote(body, env, depth)?);
                Ok(Value::List(items.into()))
            }
            Expr::Let { bindings, body, .. } => {
                let mut items = vec![Value::Symbol("let".to_string())];
                let mut binding_vec = Vec::new();
                for (pattern, expr) in bindings {
                    binding_vec.push(self.fn_param_to_value(pattern));
                    binding_vec.push(self.eval_quasiquote(expr, Arc::clone(&env), depth)?);
                }
                items.push(Value::Vector(binding_vec.into()));
                items.push(self.eval_quasiquote(body, env, depth)?);
                Ok(Value::List(items.into()))
            }
            Expr::Def {
                name,
                value,
                is_private: _is_private,
                ..
            } => {
                let mut items = vec![
                    Value::Symbol("def".to_string()),
                    Value::Symbol(name.clone()),
                ];
                items.push(self.eval_quasiquote(value, env, depth)?);
                Ok(Value::List(items.into()))
            }
            // その他は変換してValueに
            _ => self.expr_to_value(expr),
        }
    }

    /// PatternをValueに変換（マクロ展開/quote用）
    #[allow(clippy::only_used_in_recursion)]
    fn fn_param_to_value(&self, param: &crate::value::Pattern) -> Value {
        use std::collections::HashMap;
        match param {
            Pattern::Var(name) => Value::Symbol(name.clone()),
            Pattern::List(params, rest) | Pattern::Vector(params, rest) => {
                let mut items: Vec<Value> =
                    params.iter().map(|p| self.fn_param_to_value(p)).collect();
                // restがある場合は [..., "&", rest_name] の形式にする
                if let Some(rest_param) = rest {
                    items.push(Value::Symbol("&".to_string()));
                    items.push(self.fn_param_to_value(rest_param));
                }
                // Listパターンの場合はList、Vectorパターンの場合はVectorとして返す
                match param {
                    Pattern::List(_, _) => Value::List(items.into()),
                    _ => Value::Vector(items.into()),
                }
            }
            Pattern::Map(pairs, as_var) => {
                let mut map = HashMap::new();
                for (key, pattern) in pairs {
                    map.insert(key.clone(), self.fn_param_to_value(pattern));
                }
                // :as がある場合は追加
                if let Some(var) = as_var {
                    map.insert("as".to_string(), Value::Symbol(var.clone()));
                }
                Value::Map(map.into())
            }
            Pattern::As(inner, var) => {
                // (:as inner-pattern var) の形式で表現
                Value::List(
                    vec![
                        Value::Symbol("as".to_string()),
                        self.fn_param_to_value(inner),
                        Value::Symbol(var.clone()),
                    ]
                    .into(),
                )
            }
            // match専用パターン（quote/マクロでも表現）
            Pattern::Wildcard => Value::Symbol("_".to_string()),
            Pattern::Nil => Value::Nil,
            Pattern::Bool(b) => Value::Bool(*b),
            Pattern::Integer(n) => Value::Integer(*n),
            Pattern::Float(f) => Value::Float(*f),
            Pattern::String(s) => Value::String(s.clone()),
            Pattern::Keyword(k) => Value::Keyword(k.clone()),
            Pattern::Transform(var, expr) => {
                // (:transform var expr) の形式で表現
                Value::List(
                    vec![
                        Value::Symbol("transform".to_string()),
                        Value::Symbol(var.clone()),
                        self.expr_to_value(expr).unwrap_or(Value::Nil),
                    ]
                    .into(),
                )
            }
            Pattern::Or(patterns) => {
                // (:or pattern1 pattern2 ...) の形式で表現
                let mut items = vec![Value::Symbol("or".to_string())];
                items.extend(patterns.iter().map(|p| self.fn_param_to_value(p)));
                Value::List(items.into())
            }
        }
    }

    /// ExprをValueに変換（データとして扱う）
    fn expr_to_value(&self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Nil { .. } => Ok(Value::Nil),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::Integer { value, .. } => Ok(Value::Integer(*value)),
            Expr::Float { value, .. } => Ok(Value::Float(*value)),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::Symbol { name, .. } => Ok(Value::Symbol(name.clone())),
            Expr::Keyword { name, .. } => Ok(Value::Keyword(name.clone())),
            Expr::List { items, .. } => {
                let mut vals = Vec::with_capacity(items.len());
                for item in items {
                    vals.push(self.expr_to_value(item)?);
                }
                Ok(Value::List(vals.into()))
            }
            Expr::Vector { items, .. } => {
                let mut vals = Vec::with_capacity(items.len());
                for item in items {
                    vals.push(self.expr_to_value(item)?);
                }
                Ok(Value::Vector(vals.into()))
            }
            Expr::Map { pairs, .. } => {
                let mut map = HashMap::with_capacity(pairs.len());
                for (k, v) in pairs {
                    let key_value = self.expr_to_value(k)?;
                    let key = key_value.to_map_key()?;
                    let value = self.expr_to_value(v)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map.into()))
            }
            // 特殊形式やCallは評価せずにリストとして返す
            Expr::Call { func, args, .. } => {
                let mut items = vec![self.expr_to_value(func)?];
                for arg in args {
                    items.push(self.expr_to_value(arg)?);
                }
                Ok(Value::List(items.into()))
            }
            Expr::If {
                test,
                then,
                otherwise,
                ..
            } => {
                let mut items = vec![Value::Symbol("if".to_string())];
                items.push(self.expr_to_value(test)?);
                items.push(self.expr_to_value(then)?);
                if let Some(o) = otherwise {
                    items.push(self.expr_to_value(o)?);
                }
                Ok(Value::List(items.into()))
            }
            Expr::Do { exprs, .. } => {
                let mut items = vec![Value::Symbol("do".to_string())];
                for e in exprs {
                    items.push(self.expr_to_value(e)?);
                }
                Ok(Value::List(items.into()))
            }
            Expr::Def {
                name,
                value,
                is_private: _is_private,
                ..
            } => Ok(Value::List(
                vec![
                    Value::Symbol("def".to_string()),
                    Value::Symbol(name.clone()),
                    self.expr_to_value(value)?,
                ]
                .into(),
            )),
            Expr::Let { bindings, body, .. } => {
                let mut items = vec![Value::Symbol("let".to_string())];
                let mut binding_vec = Vec::new();
                for (pattern, expr) in bindings {
                    binding_vec.push(self.fn_param_to_value(pattern));
                    binding_vec.push(self.expr_to_value(expr)?);
                }
                items.push(Value::Vector(binding_vec.into()));
                items.push(self.expr_to_value(body)?);
                Ok(Value::List(items.into()))
            }
            Expr::Fn {
                params,
                body,
                is_variadic,
                ..
            } => {
                let mut items = vec![Value::Symbol("fn".to_string())];
                let param_vals: Vec<Value> = if *is_variadic && params.len() == 1 {
                    vec![
                        Value::Symbol("&".to_string()),
                        self.fn_param_to_value(&params[0]),
                    ]
                } else if *is_variadic {
                    let mut v: Vec<Value> = params[..params.len() - 1]
                        .iter()
                        .map(|p| self.fn_param_to_value(p))
                        .collect();
                    v.push(Value::Symbol("&".to_string()));
                    v.push(self.fn_param_to_value(&params[params.len() - 1]));
                    v
                } else {
                    params.iter().map(|p| self.fn_param_to_value(p)).collect()
                };
                items.push(Value::Vector(param_vals.into()));
                items.push(self.expr_to_value(body)?);
                Ok(Value::List(items.into()))
            }
            Expr::Quasiquote { expr: e, .. } => Ok(Value::List(
                vec![
                    Value::Symbol("quasiquote".to_string()),
                    self.expr_to_value(e)?,
                ]
                .into(),
            )),
            Expr::Unquote { expr: e, .. } => Ok(Value::List(
                vec![Value::Symbol("unquote".to_string()), self.expr_to_value(e)?].into(),
            )),
            Expr::UnquoteSplice { expr: e, .. } => Ok(Value::List(
                vec![
                    Value::Symbol("unquote-splice".to_string()),
                    self.expr_to_value(e)?,
                ]
                .into(),
            )),
            // モジュール関連とtry、deferはquoteできない
            Expr::Module { .. }
            | Expr::Export { .. }
            | Expr::Use { .. }
            | Expr::Try { .. }
            | Expr::Defer { .. }
            | Expr::Loop { .. }
            | Expr::Recur { .. }
            | Expr::When { .. }
            | Expr::While { .. }
            | Expr::Until { .. }
            | Expr::WhileSome { .. }
            | Expr::UntilError { .. }
            | Expr::Match { .. }
            | Expr::Mac { .. } => Err(fmt_msg(
                MsgKey::CannotQuote,
                &["module/export/use/try/defer/loop/recur/when/while/until/while-some/until-error/match/mac"],
            )),
            Expr::FString { .. } => Err(msg(MsgKey::FStringCannotBeQuoted).to_string()),
        }
    }

    /// マクロを展開
    fn expand_macro(
        &self,
        mac: &Macro,
        args: &[Expr],
        _env: Arc<RwLock<Env>>,
    ) -> Result<Expr, String> {
        // マクロ用の環境を作成
        let parent_env = Arc::clone(&mac.env);
        let mut new_env = Env::with_parent(parent_env);

        if mac.is_variadic {
            // 可変長引数の処理：最後のパラメータが可変引数
            if mac.params.is_empty() {
                return Err(msg(MsgKey::VariadicMacroNeedsParams).to_string());
            }

            let fixed_count = mac.params.len() - 1;

            // 固定引数が足りない場合エラー
            if args.len() < fixed_count {
                return Err(fmt_msg(
                    MsgKey::MacVariadicArgCountMismatch,
                    &[&mac.name, &fixed_count.to_string(), &args.len().to_string()],
                ));
            }

            // 固定引数を設定
            for (param, arg) in mac.params.iter().zip(args.iter()).take(fixed_count) {
                let arg_val = self.expr_to_value(arg)?;
                new_env.set(param.clone(), arg_val);
            }

            // 残りを可変引数として設定
            let rest: Vec<Value> = args[fixed_count..]
                .iter()
                .map(|e| self.expr_to_value(e))
                .collect::<Result<Vec<_>, _>>()?;
            new_env.set(mac.params[fixed_count].clone(), Value::List(rest.into()));
        } else {
            // 通常の引数
            if mac.params.len() != args.len() {
                return Err(fmt_msg(
                    MsgKey::MacArgCountMismatch,
                    &[
                        &mac.name,
                        &mac.params.len().to_string(),
                        &args.len().to_string(),
                    ],
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
            Value::Nil => Ok(Expr::Nil {
                span: Expr::dummy_span(),
            }),
            Value::Bool(b) => Ok(Expr::Bool {
                value: *b,
                span: Expr::dummy_span(),
            }),
            Value::Integer(n) => Ok(Expr::Integer {
                value: *n,
                span: Expr::dummy_span(),
            }),
            Value::Float(f) => Ok(Expr::Float {
                value: *f,
                span: Expr::dummy_span(),
            }),
            Value::String(s) => Ok(Expr::String {
                value: s.clone(),
                span: Expr::dummy_span(),
            }),
            Value::Symbol(s) => Ok(Expr::Symbol {
                name: s.clone(),
                span: Expr::dummy_span(),
            }),
            Value::Keyword(k) => Ok(Expr::Keyword {
                name: k.clone(),
                span: Expr::dummy_span(),
            }),
            Value::List(items) if items.is_empty() => Ok(Expr::List {
                items: vec![],
                span: Expr::dummy_span(),
            }),
            Value::List(items) => {
                // 先頭がシンボルの場合、特殊形式かチェック
                if let Some(Value::Symbol(s)) = items.head() {
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
                                span: Expr::dummy_span(),
                            });
                        }
                        "do" => {
                            let exprs: Result<Vec<_>, _> = items
                                .iter()
                                .skip(1)
                                .map(|v| self.value_to_expr(v))
                                .collect();
                            return Ok(Expr::Do {
                                exprs: exprs?,
                                span: Expr::dummy_span(),
                            });
                        }
                        "def" if items.len() == 3 || items.len() == 4 => {
                            if let Value::Symbol(name) = &items[1] {
                                // 4要素の場合: (def name "doc" value)
                                if items.len() == 4 {
                                    // items[2]がドキュメント文字列
                                    if let Value::String(doc) = &items[2] {
                                        let doc_key = format!("{}{}", DOC_PREFIX, name);
                                        self.global_env
                                            .write()
                                            .set(doc_key, Value::String(doc.clone()));
                                    } else if let Value::Map(doc_map) = &items[2] {
                                        // 構造化ドキュメント（マップ）
                                        if let Some(Value::String(desc)) = doc_map.get("desc") {
                                            let doc_key = format!("{}{}", DOC_PREFIX, name);
                                            self.global_env
                                                .write()
                                                .set(doc_key, Value::String(desc.clone()));
                                        }
                                    }
                                    // 値はitems[3]
                                    return Ok(Expr::Def {
                                        name: name.clone(),
                                        value: Box::new(self.value_to_expr(&items[3])?),
                                        is_private: false,
                                        span: Expr::dummy_span(),
                                    });
                                } else {
                                    // 3要素の場合: (def name value)
                                    return Ok(Expr::Def {
                                        name: name.clone(),
                                        value: Box::new(self.value_to_expr(&items[2])?),
                                        is_private: false,
                                        span: Expr::dummy_span(),
                                    });
                                }
                            }
                        }
                        "defn" if items.len() >= 4 => {
                            // defn展開: (defn name [params] body) -> (def name (fn [params] body))
                            // ドキュメント文字列があれば __doc__<name> に保存
                            if let Value::Symbol(name) = &items[1] {
                                // パラメータリスト（Vector）の位置を探す
                                let mut params_idx = 2;
                                let mut doc_string: Option<String> = None;

                                // items[2]がVectorでなければドキュメント
                                if !matches!(&items[2], Value::Vector(_)) {
                                    // ドキュメント文字列を抽出
                                    if let Value::String(doc) = &items[2] {
                                        doc_string = Some(doc.clone());
                                    } else if let Value::Map(doc_map) = &items[2] {
                                        // 構造化ドキュメント（マップ）もサポート
                                        // 後で実装予定。今は文字列のみ
                                        doc_string = doc_map.get("desc").and_then(|v| match v {
                                            Value::String(s) => Some(s.clone()),
                                            _ => None,
                                        });
                                    }
                                    params_idx = 3;
                                }

                                // パラメータリストと本体を確認
                                if params_idx < items.len()
                                    && matches!(&items[params_idx], Value::Vector(_))
                                {
                                    // ドキュメント文字列を保存
                                    if let Some(doc) = doc_string {
                                        let doc_key = format!("{}{}", DOC_PREFIX, name);
                                        self.global_env.write().set(doc_key, Value::String(doc));
                                    }

                                    let params = items[params_idx].clone();
                                    let body: Vec<Value> =
                                        items.iter().skip(params_idx + 1).cloned().collect();

                                    // (fn [params] body...) を構築
                                    let mut fn_items =
                                        vec![Value::Symbol("fn".to_string()), params];
                                    fn_items.extend(body);
                                    let fn_value = Value::List(fn_items.into());

                                    // (def name (fn ...)) を構築
                                    let def_items = vec![
                                        Value::Symbol("def".to_string()),
                                        Value::Symbol(name.clone()),
                                        fn_value,
                                    ];

                                    // 展開したdefを再度処理
                                    return self.value_to_expr(&Value::List(def_items.into()));
                                }
                            }
                        }
                        // quasiquote/unquote/unquote-spliceは展開後には出現しないはず
                        // もし出現した場合は通常のリストとして扱う
                        _ => {}
                    }
                }
                // 通常のリストまたは関数呼び出し
                let exprs: Result<Vec<_>, _> =
                    items.iter().map(|v| self.value_to_expr(v)).collect();
                let exprs = exprs?;

                // 先頭がシンボルの場合はCallに変換（関数呼び出しとして扱う）
                if let Some(Expr::Symbol { .. }) = exprs.first() {
                    if exprs.len() == 1 {
                        // 単一のシンボルはそのまま
                        Ok(Expr::List {
                            items: exprs,
                            span: Expr::dummy_span(),
                        })
                    } else {
                        // 関数呼び出し
                        Ok(Expr::Call {
                            func: Box::new(exprs[0].clone()),
                            args: exprs[1..].to_vec(),
                            span: Expr::dummy_span(),
                        })
                    }
                } else {
                    Ok(Expr::List {
                        items: exprs,
                        span: Expr::dummy_span(),
                    })
                }
            }
            Value::Vector(items) => {
                let exprs: Result<Vec<_>, _> =
                    items.iter().map(|v| self.value_to_expr(v)).collect();
                Ok(Expr::Vector {
                    items: exprs?,
                    span: Expr::dummy_span(),
                })
            }
            _ => Err(msg(MsgKey::ValueCannotBeConverted).to_string()),
        }
    }

    // ========================================
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval_str(s: &str) -> Result<Value, String> {
        crate::i18n::init(); // i18nシステムを初期化
        let evaluator = Evaluator::new();
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
        assert_eq!(
            eval_str("(+ (* 2 3) (- 10 5))").unwrap(),
            Value::Integer(11)
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(eval_str("(if true 1 2)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(if false 1 2)").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_fn() {
        assert_eq!(eval_str("((fn [x] (+ x 1)) 5)").unwrap(), Value::Integer(6));
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
    fn test_match_rest() {
        // ...restパターンのテスト
        assert_eq!(
            eval_str("(match [1 2 3 4 5] [x ...rest] -> rest)").unwrap(),
            Value::List(
                vec![
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4),
                    Value::Integer(5)
                ]
                .into()
            )
        );
        // 1要素の場合
        assert_eq!(
            eval_str("(match [1] [x ...rest] -> rest)").unwrap(),
            Value::List(vec![].into())
        );
        // 空リストの場合
        assert_eq!(
            eval_str("(match [] [...rest] -> rest)").unwrap(),
            Value::List(vec![].into())
        );
        // 複数要素を取得してからrest
        assert_eq!(
            eval_str("(match [10 20 30] [a b ...rest] -> rest)").unwrap(),
            Value::List(vec![Value::Integer(30)].into())
        );
        // リストでも動作
        assert_eq!(
            eval_str("(match (list 1 2 3) [x ...rest] -> rest)").unwrap(),
            Value::List(vec![Value::Integer(2), Value::Integer(3)].into())
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
        assert_eq!(eval_str("(10 |> (+ 5))").unwrap(), Value::Integer(15));
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
            Value::List(vec![Value::Integer(2), Value::Integer(4), Value::Integer(6)].into())
        );
    }

    #[test]
    fn test_filter() {
        // filterのテスト
        assert_eq!(
            eval_str("(filter (fn [x] (> x 2)) [1 2 3 4 5])").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(4), Value::Integer(5)].into())
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
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into())
        );
        assert_eq!(
            eval_str("(cons 1 nil)").unwrap(),
            Value::List(vec![Value::Integer(1)].into())
        );
    }

    #[test]
    fn test_conj() {
        // conjのテスト
        assert_eq!(
            eval_str("(conj [1 2] 3 4)").unwrap(),
            Value::Vector(
                vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4)
                ]
                .into()
            )
        );
        assert_eq!(
            eval_str("(conj (list 1 2) 3)").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(1), Value::Integer(2)].into())
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
            Value::List(vec![Value::Integer(6), Value::Integer(8), Value::Integer(10)].into())
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
        assert_eq!(eval_str("'x").unwrap(), Value::Symbol("x".to_string()));
        assert_eq!(
            eval_str("'(1 2 3)").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into())
        );
        assert_eq!(
            eval_str("'(+ 1 2)").unwrap(),
            Value::List(
                vec![
                    Value::Symbol("+".to_string()),
                    Value::Integer(1),
                    Value::Integer(2)
                ]
                .into()
            )
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
        // 成功時は生の値を返す（{:ok value}ではない）
        let result = eval_str("(try (+ 1 2))").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_try_error() {
        // エラー時は {:error msg}
        let result = eval_str("(try (/ 1 0))").unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.get(":ok"), None);
                assert!(m.get(":error").is_some());
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_try_with_match() {
        // tryとmatchの組み合わせ
        // 成功時は生の値が返される
        let result = eval_str(
            r#"
            (match (try (+ 1 2))
              {:error e} -> 0
              result -> result)
            "#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(3));

        // エラー時は {:error ...} が返される
        let result = eval_str(
            r#"
            (match (try (/ 1 0))
              {:error e} -> -1
              result -> result)
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
            Value::Vector(vec![Value::Integer(3), Value::Integer(2), Value::Integer(1)].into())
        );
        assert_eq!(
            eval_str("(reverse '(a b c))").unwrap(),
            Value::List(
                vec![
                    Value::Symbol("c".to_string()),
                    Value::Symbol("b".to_string()),
                    Value::Symbol("a".to_string())
                ]
                .into()
            )
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

    #[test]
    fn test_tap() {
        // tap関数が値を返しつつ副作用を実行することを確認
        // （副作用のテストは難しいので、値が正しく返されることのみ確認）
        assert_eq!(
            eval_str("([1 2 3] |> (map inc) |> (tap (fn [x] x)) |> sum)").unwrap(),
            Value::Integer(9)
        );

        // tapが元の値をそのまま返すことを確認
        assert_eq!(
            eval_str("(def x 42) (x |> (tap (fn [y] (+ y 1))))").unwrap(),
            Value::Integer(42) // 副作用の結果ではなく元の値
        );

        // fn/tap>（高階関数版）のテスト
        assert_eq!(
            eval_str("([1 2 3] |> (map inc) |> ((fn/tap> (fn [x] x))) |> sum)").unwrap(),
            Value::Integer(9)
        );
    }

    #[test]
    fn test_match_or_pattern() {
        // orパターンのテスト - 数値
        assert_eq!(
            eval_str("(match 1 1 | 2 | 3 -> \"small\" _ -> \"large\")").unwrap(),
            Value::String("small".to_string())
        );
        assert_eq!(
            eval_str("(match 2 1 | 2 | 3 -> \"small\" _ -> \"large\")").unwrap(),
            Value::String("small".to_string())
        );
        assert_eq!(
            eval_str("(match 3 1 | 2 | 3 -> \"small\" _ -> \"large\")").unwrap(),
            Value::String("small".to_string())
        );
        assert_eq!(
            eval_str("(match 5 1 | 2 | 3 -> \"small\" _ -> \"large\")").unwrap(),
            Value::String("large".to_string())
        );

        // orパターンのテスト - 文字列
        assert_eq!(
            eval_str("(match \"red\" \"red\" | \"blue\" -> \"primary\" _ -> \"other\")").unwrap(),
            Value::String("primary".to_string())
        );
        assert_eq!(
            eval_str("(match \"blue\" \"red\" | \"blue\" -> \"primary\" _ -> \"other\")").unwrap(),
            Value::String("primary".to_string())
        );
        assert_eq!(
            eval_str("(match \"green\" \"red\" | \"blue\" -> \"primary\" _ -> \"other\")").unwrap(),
            Value::String("other".to_string())
        );

        // orパターンのテスト - 変数バインディング付き
        assert_eq!(
            eval_str("(match 2 1 | 2 | 3 -> (+ 10 2) _ -> 0)").unwrap(),
            Value::Integer(12)
        );
    }

    #[test]
    fn test_match_or_pattern_with_wildcards() {
        // orパターン + ワイルドカード
        assert_eq!(
            eval_str("(match nil nil | false -> \"falsy\" _ -> \"truthy\")").unwrap(),
            Value::String("falsy".to_string())
        );
        assert_eq!(
            eval_str("(match false nil | false -> \"falsy\" _ -> \"truthy\")").unwrap(),
            Value::String("falsy".to_string())
        );
        assert_eq!(
            eval_str("(match true nil | false -> \"falsy\" _ -> \"truthy\")").unwrap(),
            Value::String("truthy".to_string())
        );
    }

    #[test]
    fn test_use_as_alias() {
        // 一時的なモジュールファイルを作成
        use std::fs;

        let module_path = "/tmp/test_alias_module.qi";
        let test_path = "/tmp/test_alias.qi";

        // モジュールファイルを作成
        fs::write(
            module_path,
            r#"
(module test_alias_module)
(def double (fn [x] (* x 2)))
(def triple (fn [x] (* x 3)))
(export double triple)
"#,
        )
        .unwrap();

        // テストファイルを作成
        fs::write(
            test_path,
            r#"
(use test_alias_module :as tm)
(+ (tm/double 5) (tm/triple 3))
"#,
        )
        .unwrap();

        // 評価
        let content = fs::read_to_string(test_path).unwrap();
        let result = std::panic::catch_unwind(|| {
            let evaluator = Evaluator::new();
            let mut parser = crate::parser::Parser::new(&content).unwrap();
            let exprs = parser.parse_all().unwrap();
            let mut last = Value::Nil;

            // /tmp ディレクトリに移動（モジュールロードのため）
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir("/tmp").unwrap();

            for expr in exprs {
                last = evaluator.eval(&expr).unwrap();
            }

            // 元のディレクトリに戻る
            std::env::set_current_dir(original_dir).unwrap();
            last
        });

        // クリーンアップ
        let _ = fs::remove_file(module_path);
        let _ = fs::remove_file(test_path);

        // 結果確認: (tm/double 5) = 10, (tm/triple 3) = 9, 10 + 9 = 19
        assert_eq!(result.unwrap(), Value::Integer(19));
    }
}
