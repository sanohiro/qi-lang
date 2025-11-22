use crate::builtins;
use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::lexer::Span;
use crate::value::{Env, Expr, Function, Module, NativeFunc, Value};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

// ========================================
// サブモジュール
// ========================================

pub mod call;
pub mod core;
pub mod helpers;
mod modules;
mod patterns;
mod special_forms;

// helpersとcoreからの関数をインポート
use core::{
    native_is_fn, native_is_number, native_list, native_print, native_to_list, native_to_vector,
    native_vector,
};
use helpers::{find_similar_names, qerr};

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

pub struct Evaluator {
    global_env: Arc<RwLock<Env>>,
    defer_stack: Arc<RwLock<SmallVec<[Vec<Expr>; 1]>>>, // スコープごとのdeferスタック（ほとんど0～1個）
    modules: Arc<RwLock<HashMap<Arc<str>, Arc<Module>>>>, // ロード済みモジュール（Arc<str>で統一、後方互換性のため残す）
    module_states: Arc<dashmap::DashMap<Arc<str>, crate::value::ModuleState>>, // モジュール状態管理（スレッド間の循環検出、アトミック操作）
    current_module: Arc<RwLock<Option<Arc<str>>>>, // 現在評価中のモジュール名
    loading_modules: Arc<RwLock<Vec<Arc<str>>>>,   // exportキー用スタック（スレッドローカル）
    #[allow(dead_code)]
    call_stack: Arc<RwLock<Vec<String>>>, // 関数呼び出しスタック（スタックトレース用）
    source_name: Arc<RwLock<Option<String>>>,      // ソースファイル名または入力名
    source_code: Arc<RwLock<Option<String>>>,      // ソースコード全体
}

impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Self {
            // グローバル状態は共有
            global_env: Arc::clone(&self.global_env),
            modules: Arc::clone(&self.modules),
            // 循環検出の仕組み:
            // - module_states: スレッド間で共有され、アトミックな状態管理（Loaded/Loading）
            // - loading_modules: スレッドローカルで、各スレッドの依存パスを追跡
            // この設計により、スレッド間の循環検出とエラーメッセージ構築が両立する
            module_states: Arc::clone(&self.module_states), // スレッド間の循環検出用（共有）

            // 評価コンテキストは独立（新しいインスタンスを作成）
            defer_stack: Arc::new(RwLock::new(Default::default())),
            loading_modules: Arc::new(RwLock::new(Default::default())), // スレッドローカル（意図的）
            current_module: Arc::new(RwLock::new(Default::default())),
            call_stack: Arc::new(RwLock::new(Default::default())),
            source_name: Arc::new(RwLock::new(Default::default())),
            source_code: Arc::new(RwLock::new(Default::default())),
        }
    }
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
            "print",
            Value::NativeFunc(NativeFunc {
                name: "print",
                func: native_print,
            }),
        );
        env_rc.write().set(
            "list",
            Value::NativeFunc(NativeFunc {
                name: "list",
                func: native_list,
            }),
        );
        env_rc.write().set(
            "vector",
            Value::NativeFunc(NativeFunc {
                name: "vector",
                func: native_vector,
            }),
        );
        env_rc.write().set(
            "to-list",
            Value::NativeFunc(NativeFunc {
                name: "to-list",
                func: native_to_list,
            }),
        );
        env_rc.write().set(
            "to-vector",
            Value::NativeFunc(NativeFunc {
                name: "to-vector",
                func: native_to_vector,
            }),
        );

        // 型判定関数（builtins以外のもの）
        env_rc.write().set(
            "number?",
            Value::NativeFunc(NativeFunc {
                name: "number?",
                func: native_is_number,
            }),
        );
        env_rc.write().set(
            "fn?",
            Value::NativeFunc(NativeFunc {
                name: "fn?",
                func: native_is_fn,
            }),
        );

        let evaluator = Evaluator {
            global_env: env_rc.clone(),
            defer_stack: Arc::new(RwLock::new(SmallVec::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
            module_states: Arc::new(dashmap::DashMap::new()),
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
            Expr::Keyword { name, .. } => {
                // インターン化でメモリ削減・比較高速化
                Ok(Value::Keyword(crate::intern::intern_keyword(name)))
            }

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
                // im::Vector は FromIterator を実装しているので直接 collect
                // Vec → im::Vector の二重アロケーションを回避
                let values: im::Vector<Value> = items
                    .iter()
                    .map(|item| self.eval_with_env(item, Arc::clone(&env)))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(values))
            }

            Expr::Vector { items, .. } => {
                // im::Vector は FromIterator を実装しているので直接 collect
                // Vec → im::Vector の二重アロケーションを回避
                let values: im::Vector<Value> = items
                    .iter()
                    .map(|item| self.eval_with_env(item, Arc::clone(&env)))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Vector(values))
            }

            Expr::Map { pairs, .. } => {
                // ahashを使ったHashMapを最初から構築（std::HashMapからの変換を避ける）
                let mut map = crate::new_hashmap();
                for (k, v) in pairs {
                    let key_value = self.eval_with_env(k, Arc::clone(&env))?;
                    let key = key_value.to_map_key()?;
                    let value = self.eval_with_env(v, Arc::clone(&env))?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
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
                body: Arc::new((**body).clone()),
                env: Arc::clone(&env),
                is_variadic: *is_variadic,
                has_special_processing: false,
            }))),

            Expr::Let { bindings, body, .. } => {
                // let*セマンティクス: 順次束縛（各束縛が前の束縛を参照できる）
                let new_env = Arc::new(RwLock::new(Env::with_parent(Arc::clone(&env))));
                for (pattern, expr) in bindings {
                    // 現在の環境で評価（既に追加された束縛が見える）
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
                    // ループ専用の環境を作成（束縛のリークを防ぐ）
                    let loop_env = Arc::new(RwLock::new(Env::with_parent(Arc::clone(&env))));
                    loop_env.write().set(binding.clone(), val);
                    self.eval_do(body, loop_env)?;
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
                        if m.contains_key(&crate::constants::keywords::error_mapkey()) {
                            return Ok(val);
                        }
                    }

                    // ループ専用の環境を作成（束縛のリークを防ぐ）
                    let loop_env = Arc::new(RwLock::new(Env::with_parent(Arc::clone(&env))));
                    loop_env.write().set(binding.clone(), val);
                    self.eval_do(body, loop_env)?;
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
                // 現在ロード中のファイル名を取得（フォールバック用）
                let file_path = self
                    .loading_modules
                    .read()
                    .last()
                    .cloned()
                    .ok_or_else(|| msg(MsgKey::ExportOnlyInModule).to_string())?;

                // モジュール名をキーとして優先、なければファイルパスを使用
                let module_key = self
                    .current_module
                    .read()
                    .clone()
                    .unwrap_or_else(|| file_path.clone());

                let module_display_name = module_key.clone();

                // シンボルの存在確認
                for symbol in symbols {
                    if env.read().get(symbol).is_none() {
                        return Err(qerr(
                            MsgKey::SymbolNotFound,
                            &[symbol, &module_display_name],
                        ));
                    }
                }

                // 既存のモジュールを取得または新規作成
                let mut modules = self.modules.write();
                if let Some(existing_module) = modules.get(&module_key) {
                    // 既存のモジュールがある場合は、exportsを累積
                    let mut new_exports = existing_module
                        .exports
                        .clone()
                        .unwrap_or_else(crate::new_hashset);
                    new_exports.extend(symbols.iter().cloned());

                    let updated_module = Module {
                        name: module_display_name,
                        file_path: existing_module.file_path.clone(), // 既存のfile_pathを保持
                        env: Arc::clone(&env),
                        exports: Some(new_exports),
                    };
                    modules.insert(module_key, Arc::new(updated_module));
                } else {
                    // 新規モジュールの場合
                    let module = Module {
                        name: module_display_name,
                        file_path: Some(file_path.to_string()),
                        env: Arc::clone(&env),
                        exports: Some(symbols.iter().cloned().collect()),
                    };
                    modules.insert(module_key, Arc::new(module));
                }

                Ok(Value::Nil)
            }

            Expr::Use { module, mode, .. } => self.eval_use(module, mode, env),

            Expr::Call { func, args, .. } => self.eval_call(func, args, env),
        }
    }

    /// 特殊形式（高階関数、演算子など）のディスパッチ
    ///
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

    fn eval_source(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        let expr = self.eval_with_env(&args[0], Arc::clone(&env))?;
        builtins::source(&[expr], self)
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

    /// table/where - テーブルの行をフィルタリング
    fn eval_table_where(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["table/where"]));
        }
        let table = self.eval_with_env(&args[0], Arc::clone(&env))?;
        let predicate = self.eval_with_env(&args[1], Arc::clone(&env))?;
        builtins::table_where(&[table, predicate], self)
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
                    body: Arc::new(expr.clone()),
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
            return Err(qerr(MsgKey::Need2Args, &["map/update-keys"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::map_update_keys(&vals, self)
    }

    fn eval_update_vals(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["map/update-vals"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::map_update_vals(&vals, self)
    }

    fn eval_map_filter_vals(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["map/filter-vals"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::map_filter_vals(&vals, self)
    }

    fn eval_map_group_by(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(qerr(MsgKey::Need2Args, &["map/group-by"]));
        }
        let vals: Vec<Value> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<Vec<_>, _>>()?;
        builtins::map_group_by(&vals, self)
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
            Value::Symbol(crate::intern::intern_symbol("x"))
        );
        assert_eq!(
            eval_str("'x").unwrap(),
            Value::Symbol(crate::intern::intern_symbol("x"))
        );
        assert_eq!(
            eval_str("'(1 2 3)").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into())
        );
        assert_eq!(
            eval_str("'(+ 1 2)").unwrap(),
            Value::List(
                vec![
                    Value::Symbol(crate::intern::intern_symbol("+")),
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
                assert_eq!(m.get(&crate::constants::keywords::ok_mapkey()), None);
                assert!(m.get(&crate::constants::keywords::error_mapkey()).is_some());
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
                    Value::Symbol(crate::intern::intern_symbol("c")),
                    Value::Symbol(crate::intern::intern_symbol("b")),
                    Value::Symbol(crate::intern::intern_symbol("a"))
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
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let module_path = temp_dir.join("module_alias.qi");

        // モジュールファイルを作成
        fs::write(
            &module_path,
            r#"
(module module_alias)
(def double (fn [x] (* x 2)))
(def triple (fn [x] (* x 3)))
(export double triple)
"#,
        )
        .unwrap();

        // Windows短縮形パス（~1など）およびverbatim prefix（\\?\）を正規化
        let module_path = dunce::canonicalize(&module_path).unwrap();
        let test_path = temp_dir.join("test_alias.qi");

        // テストファイルを作成（モジュール名のみ指定、クロスプラットフォーム対応）
        let use_statement = r#"
(use module_alias :as tm)
(+ (tm/double 5) (tm/triple 3))
"#;

        fs::write(&test_path, use_statement).unwrap();

        // test_pathもWindows短縮形パス・verbatim prefixを正規化
        let test_path = dunce::canonicalize(&test_path).unwrap();

        // 評価
        let content = fs::read_to_string(&test_path).unwrap();
        let test_path_str = test_path.to_string_lossy().to_string();

        let result = std::panic::catch_unwind(move || {
            let evaluator = Evaluator::new();
            // source_nameを設定してモジュール解決パスを正しく認識させる
            let mut parser = crate::parser::Parser::new(&content).unwrap();
            parser.set_source_name(test_path_str.clone());
            evaluator.set_source(test_path_str.clone(), content.clone());

            let exprs = parser.parse_all().unwrap();
            let mut last = Value::Nil;

            for expr in exprs {
                last = evaluator.eval(&expr).unwrap();
            }

            last
        });

        // クリーンアップ
        let _ = fs::remove_file(module_path);
        let _ = fs::remove_file(test_path);

        // 結果確認: (tm/double 5) = 10, (tm/triple 3) = 9, 10 + 9 = 19
        assert_eq!(result.unwrap(), Value::Integer(19));
    }

    #[test]
    fn test_defer_executes_on_normal_exit() {
        // deferは正常終了時に実行される
        let code = r#"
(def x (atom 0))
(do
  (defer (swap! x (fn [n] (+ n 10))))
  (defer (swap! x (fn [n] (+ n 1))))
  (swap! x (fn [n] (+ n 100))))
@x  ;; 100 + 1 + 10 = 111 (LIFO順)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::Integer(111));
    }

    #[test]
    fn test_defer_executes_on_error() {
        // deferはエラー発生時も実行される（RAII）
        let code = r#"
(def x (atom 0))
(match (try
  (do
    (defer (swap! x (fn [n] (+ n 10))))
    (defer (swap! x (fn [n] (+ n 1))))
    (/ 1 0)))  ;; エラー発生
  {:error _} -> @x
  result -> result)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::Integer(11)); // 0 + 1 + 10 = 11
    }

    #[test]
    fn test_defer_lifo_order() {
        // deferはLIFO（後入れ先出し）順で実行される
        let code = r#"
(def result (atom []))
(do
  (defer (swap! result (fn [v] (conj v 1))))
  (defer (swap! result (fn [v] (conj v 2))))
  (defer (swap! result (fn [v] (conj v 3)))))
@result
"#;
        let result = eval_str(code).unwrap();
        // LIFO順: 3, 2, 1
        match result {
            Value::Vector(v) => {
                assert_eq!(v.len(), 3);
                assert_eq!(v[0], Value::Integer(3));
                assert_eq!(v[1], Value::Integer(2));
                assert_eq!(v[2], Value::Integer(1));
            }
            _ => panic!("Expected vector"),
        }
    }

    #[test]
    fn test_nested_defer_scopes() {
        // ネストしたdoブロックでdeferが正しく動作する
        let code = r#"
(def x (atom 0))
(do
  (defer (swap! x (fn [n] (+ n 100))))
  (do
    (defer (swap! x (fn [n] (+ n 10))))
    (swap! x (fn [n] (+ n 1))))  ;; 内側: 1 + 10 = 11
  (swap! x (fn [n] (+ n 1000))))  ;; 外側: 11 + 1000 = 1011, then + 100 = 1111
@x
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::Integer(1111));
    }
}
