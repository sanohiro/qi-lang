//! 関数呼び出しの評価
//!
//! try_eval_special_form, eval_call, apply_function, apply_func等の
//! 関数呼び出しに関する評価ロジックを提供します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Env, Expr, Pattern, Value};
use parking_lot::RwLock;
use smallvec::{smallvec, SmallVec};
use std::sync::Arc;

use super::builtins;
use super::helpers::qerr;
use super::hof_keys;
use super::Evaluator;

/// マップキーを作成
fn to_map_key(key: &str) -> crate::value::MapKey {
    let key_str = key.strip_prefix(':').unwrap_or(key);
    crate::value::MapKey::Keyword(crate::intern::intern_keyword(key_str))
}

impl Evaluator {
    /// 特殊形式のチェック（高階関数、論理演算子、並行処理など）
    pub(super) fn try_eval_special_form(
        &self,
        func: &Expr,
        args: &[Expr],
        env: Arc<RwLock<Env>>,
    ) -> Option<Result<Value, String>> {
        if let Expr::Symbol { name, .. } = func {
            match &**name {
                "_railway-pipe" => Some(self.eval_railway_pipe(args, env)),
                "and" => Some(self.eval_and(args, env)),
                "apply" => Some(self.eval_apply(args, env)),
                "comment" => Some(Ok(Value::Nil)),
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
                "source" => Some(self.eval_source(args, env)),
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
                "map/filter-vals" => Some(self.eval_map_filter_vals(args, env)),
                "map/group-by" => Some(self.eval_map_group_by(args, env)),
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
                "table/where" => Some(self.eval_table_where(args, env)),
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
    pub(super) fn eval_call(
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
                    Value::Map(m) => {
                        // キーワードをマップキーに変換（:付き）
                        let map_key = Value::Keyword(key.clone()).to_map_key()?;
                        m.get(&map_key)
                            .cloned()
                            .ok_or_else(|| qerr(MsgKey::KeyNotFound, &[&key]))
                    }
                    _ => Err(qerr(MsgKey::TypeOnly, &["keyword fn", "maps"])),
                }
            }
            Value::String(key) => {
                // 文字列を関数として使う: ("name" map) => (get map "name")
                // JSON等の文字列キーへのアクセスに便利
                if arg_vals.len() != 1 {
                    return Err(qerr(MsgKey::NeedExactlyNArgs, &["string", "1"]));
                }
                match &arg_vals[0] {
                    Value::Map(m) => {
                        // 文字列をマップキーに変換
                        let map_key = Value::String(key.clone()).to_map_key()?;
                        m.get(&map_key)
                            .cloned()
                            .ok_or_else(|| qerr(MsgKey::KeyNotFound, &[&key]))
                    }
                    _ => Err(qerr(MsgKey::TypeOnly, &["string fn", "maps"])),
                }
            }
            _ => Err(format!(
                "{} は呼び出し可能ではありません",
                func_val.type_name()
            )),
        }
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
    pub(super) fn get_function_name(&self, func: &Value) -> Option<String> {
        if let Value::Function(f) = func {
            // グローバル環境から関数を検索
            for (name, value) in self.global_env.read().bindings() {
                if let Value::Function(stored_f) = value {
                    if Arc::ptr_eq(f, stored_f) {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    /// Patternパターンを値にマッチさせて環境にバインド
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn bind_fn_param(
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
                            &[value.type_name()],
                        ));
                    }
                };

                // restパターンがある場合とない場合で処理を分岐
                if let Some(rest) = rest_param {
                    // [x y ...rest] 形式
                    if values.len() < params.len() {
                        return Err(fmt_msg(
                            MsgKey::ArgErrorVectorPatternMinimum,
                            &[&params.len().to_string(), &values.len().to_string()],
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
                            &[&params.len().to_string(), &values.len().to_string()],
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
                        return Err(fmt_msg(MsgKey::TypeErrorMapPattern, &[value.type_name()]));
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
            Pattern::Wildcard
            | Pattern::Nil
            | Pattern::Bool(_)
            | Pattern::Integer(_)
            | Pattern::Float(_)
            | Pattern::String(_)
            | Pattern::Keyword(_)
            | Pattern::Transform(_, _)
            | Pattern::Or(_) => Err(fmt_msg(MsgKey::PatternErrorNotAllowed, &[])),
        }
    }

    /// 関数を適用するヘルパー（内部用）
    pub(super) fn apply_func(
        &self,
        func: &Value,
        args: SmallVec<[Value; 4]>,
    ) -> Result<Value, String> {
        // トレース機能: 関数呼び出しをログ出力
        // デッドロック防止: 関数名取得前にロックを解放
        let should_trace = {
            let traced_funcs = crate::builtins::debug::TRACED_FUNCTIONS.read();
            !traced_funcs.is_empty()
        };

        if should_trace {
            // 関数名を取得（global_envのロックを取得）
            if let Some(func_name) = self.get_function_name(func) {
                // 再度ロックを取得してチェック
                let traced_funcs = crate::builtins::debug::TRACED_FUNCTIONS.read();
                if traced_funcs.contains(&func_name) {
                    let args_str: Vec<String> = args.iter().map(|a| format!("{:?}", a)).collect();
                    let trace_msg = format!("→ {}({})", func_name, args_str.join(", "));

                    #[cfg(feature = "repl")]
                    {
                        use colored::Colorize;
                        eprintln!("{}", trace_msg.cyan());
                    }
                    #[cfg(not(feature = "repl"))]
                    eprintln!("{}", trace_msg);
                }
            }
        }

        // 関数を実行
        match func {
            Value::NativeFunc(nf) => (nf.func)(&args),
            Value::Function(f) => {
                // 特殊処理フラグがtrueの場合のみ環境ルックアップ（99.9%の通常関数で高速化）
                if f.has_special_processing {
                    // ロックは一度だけ取得し、必要な値をローカルにclone（ロック競合を削減）
                    let env_guard = f.env.read();
                    let complement_func = env_guard.get(hof_keys::COMPLEMENT_FUNC);
                    let juxt_funcs = env_guard.get(hof_keys::JUXT_FUNCS);
                    let tap_func = env_guard.get(hof_keys::TAP_FUNC);
                    let partial_func = env_guard.get(hof_keys::PARTIAL_FUNC);
                    let partial_args = env_guard.get(hof_keys::PARTIAL_ARGS);
                    let comp_funcs = env_guard.get(hof_keys::COMP_FUNCS);
                    drop(env_guard); // 明示的に解放

                    // complement特殊処理 - 実行前にチェック
                    if let Some(complement_func) = complement_func {
                        let result = self.apply_func(&complement_func, args)?;
                        return Ok(Value::Bool(!result.is_truthy()));
                    }

                    // juxt特殊処理 - 実行前にチェック
                    if let Some(Value::List(juxt_funcs)) = juxt_funcs {
                        let mut results: SmallVec<[Value; 4]> =
                            SmallVec::with_capacity(juxt_funcs.len());
                        for jfunc in &juxt_funcs {
                            let result = self.apply_func(jfunc, args.clone())?;
                            results.push(result);
                        }
                        return Ok(Value::Vector(results.into_iter().collect()));
                    }

                    // tap>特殊処理 - 副作用を実行してから値を返す
                    if let Some(tap_func) = tap_func {
                        if args.len() == 1 {
                            let value = args[0].clone();
                            // 副作用関数を実行（結果は無視）
                            let _ = self.apply_func(&tap_func, smallvec![value.clone()]);
                            // 元の値をそのまま返す
                            return Ok(value);
                        }
                    }

                    // partial特殊処理 - 部分適用された引数と新しい引数を結合
                    if let Some(partial_func) = partial_func {
                        if let Some(Value::List(partial_args)) = partial_args {
                            // 部分適用された引数と新しい引数を結合
                            let mut combined_args: SmallVec<[Value; 4]> =
                                SmallVec::with_capacity(partial_args.len() + args.len());
                            combined_args.extend(partial_args.iter().cloned());
                            combined_args.extend(args.iter().cloned());
                            return self.apply_func(&partial_func, combined_args);
                        }
                    }

                    // comp特殊処理 - 関数合成（右から左に適用）
                    if let Some(Value::List(comp_funcs)) = comp_funcs {
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
                    let remaining_args: SmallVec<[Value; 4]> =
                        args.iter().skip(fixed_param_count).cloned().collect();

                    if let crate::value::Pattern::Var(name) = variadic_param {
                        new_env.set(
                            name.to_string(),
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
            _ => Err(format!("{} は呼び出し可能ではありません", func.type_name())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::value::Value;

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
    fn test_keyword_as_function() {
        // キーワードキーのマップ作成とアクセス
        let code = r#"
(def response {:status 200 :body "Hello"})
(:status response)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::Integer(200));

        // 別のキーへのアクセス
        let code = r#"
(def response {:status 200 :body "Hello"})
(:body response)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));
    }

    #[test]
    fn test_string_as_function() {
        // 文字列キーのマップ作成とアクセス
        let code = r#"
(def user {"name" "Alice" "age" "30"})
("name" user)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));

        // 別のキーへのアクセス
        let code = r#"
(def user {"name" "Alice" "age" "30"})
("age" user)
"#;
        let result = eval_str(code).unwrap();
        assert_eq!(result, Value::String("30".to_string()));
    }

    #[test]
    fn test_keyword_function_with_nonexistent_key() {
        // 存在しないキーへのアクセスはエラー
        let code = r#"
(def m {})
(:nonexistent m)
"#;
        let result = eval_str(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_function_with_non_map() {
        // マップ以外に対する適用はエラー
        let code = r#"
(def x "not a map")
(:key x)
"#;
        let result = eval_str(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_function_with_non_map() {
        // マップ以外に対する適用はエラー
        let code = r#"
(def x 123)
("key" x)
"#;
        let result = eval_str(code);
        assert!(result.is_err());
    }
}
