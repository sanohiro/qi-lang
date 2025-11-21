//! 特殊形式の評価
//!
//! def, fn, let, if, do, try, defer, loop, recur, mac, quasiquote等の
//! 特殊形式の評価ロジックを提供します。

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::{Env, Expr, Macro, Value};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use super::{Evaluator, DOC_PREFIX, RECUR_SENTINEL};

// recurで評価済みの引数を一時保存するThreadLocalスタック
// loop/recurの間でのみ使用し、二重評価を回避する
// スタック構造により、入れ子のloopやEvaluatorでも正しく動作する
// Option<Vec>で「recurが呼ばれていない」と「ゼロ引数recur」を区別
thread_local! {
    static RECUR_STACK: RefCell<Vec<Option<Vec<Value>>>> = const { RefCell::new(Vec::new()) };
}

/// RAIIガード: Drop時に必ずdeferスタックをクリーンアップ
struct DeferGuard<'a> {
    evaluator: &'a Evaluator,
    env: Arc<RwLock<Env>>,
}

impl<'a> Drop for DeferGuard<'a> {
    fn drop(&mut self) {
        // deferを実行（LIFO順）
        if let Some(defers) = self.evaluator.defer_stack.write().pop() {
            for defer_expr in defers.iter().rev() {
                // deferの実行中のエラーは無視（Lisp/Rust文化に則る）
                let _ = self
                    .evaluator
                    .eval_with_env(defer_expr, Arc::clone(&self.env));
            }
        }
    }
}

/// RAIIガード: Drop時に必ずRECUR_STACKからpop
struct RecurGuard;

impl Drop for RecurGuard {
    fn drop(&mut self) {
        RECUR_STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

impl Evaluator {
    /// quote式を評価
    pub(super) fn eval_quote(&self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(qerr(MsgKey::Need1Arg, &["quote"]));
        }
        self.expr_to_value(&args[0])
    }

    /// do式を評価
    pub(super) fn eval_do(&self, exprs: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // deferスコープを作成し、RAIIガードで確実にクリーンアップ
        self.defer_stack.write().push(Vec::new());

        // RAIIガード: Drop時に必ずdeferを実行
        let _guard = DeferGuard {
            evaluator: self,
            env: Arc::clone(&env),
        };

        let mut result = Value::Nil;
        for expr in exprs {
            result = self.eval_with_env(expr, Arc::clone(&env))?;
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
    pub(super) fn eval_try(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // Tryもdeferスコープを作成
        self.defer_stack.write().push(Vec::new());

        // RAIIガード: Drop時に必ずdeferを実行（パニック時も安全）
        let _guard = DeferGuard {
            evaluator: self,
            env: Arc::clone(&env),
        };

        // エラーをキャッチして {:error} にラップ
        match self.eval_with_env(expr, Arc::clone(&env)) {
            Ok(value) => Ok(value), // :okラップなし！
            Err(e) => Ok(Value::error(e)),
        }
    }

    /// deferを評価
    pub(super) fn eval_defer(&self, expr: &Expr) -> Result<Value, String> {
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
    pub(super) fn eval_recur(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
        // 引数を評価（一度だけ）
        let values: Result<Vec<_>, _> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect();
        let values = values?;

        // 評価済みの値をThreadLocalスタックに保存（Vecを再利用）
        RECUR_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            if let Some(last) = stack.last_mut() {
                if let Some(existing_vec) = last.as_mut() {
                    // 既存のVecを再利用（キャパシティ保持）
                    existing_vec.clear();
                    existing_vec.extend(values);
                } else {
                    // 新規作成
                    *last = Some(values);
                }
            } else {
                // スタックが空の場合はエラー（loopの外でrecurが呼ばれた）
                return Err(msg(MsgKey::RecurNotFound).to_string());
            }
            Ok(())
        })?;

        // Recurシグナルをエラーとして返す
        Err(RECUR_SENTINEL.to_string())
    }

    /// macroを評価
    pub(super) fn eval_mac(
        &self,
        name: &str,
        params: &[std::sync::Arc<str>],
        is_variadic: bool,
        body: &Expr,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        let mac = Macro {
            name: Arc::from(name),
            params: params.to_vec(),
            body: Arc::new(body.clone()),
            env: Arc::clone(&env),
            is_variadic,
        };
        env.write().set(name, Value::Macro(Arc::new(mac)));
        Ok(Value::Symbol(crate::intern::intern_symbol(name)))
    }

    /// loopを評価
    pub(super) fn eval_loop(
        &self,
        bindings: &[(std::sync::Arc<str>, Expr)],
        body: &Expr,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        // RECURスタックにエントリを追加（RAIIガードで自動削除）
        RECUR_STACK.with(|s| s.borrow_mut().push(None));
        let _recur_guard = RecurGuard;

        // ループ用の環境を作成
        let mut loop_env = Env::with_parent(Arc::clone(&env));

        // 初期値で環境を設定（Iterator で一度に評価）
        let current_values: Vec<Value> = bindings
            .iter()
            .map(|(_name, expr)| self.eval_with_env(expr, Arc::clone(&env)))
            .collect::<Result<_, _>>()?;

        // 環境に設定
        for ((name, _), value) in bindings.iter().zip(current_values.iter()) {
            loop_env.set(name.clone(), value.clone());
        }

        let loop_env_rc = Arc::new(RwLock::new(loop_env));

        // ループ本体を繰り返し評価
        loop {
            match self.eval_with_env(body, loop_env_rc.clone()) {
                Ok(value) => return Ok(value),
                Err(e) if e == RECUR_SENTINEL => {
                    // Recurシグナルを検出 - ThreadLocalスタックから評価済みの値を取得
                    let new_values = RECUR_STACK
                        .with(|s| s.borrow_mut().last_mut().and_then(|v| v.take()))
                        .ok_or_else(|| msg(MsgKey::RecurNotFound).to_string())?;

                    // 引数の数をチェック
                    if bindings.len() != new_values.len() {
                        return Err(fmt_msg(
                            MsgKey::RecurArgCountMismatch,
                            &[&bindings.len().to_string(), &new_values.len().to_string()],
                        ));
                    }

                    // 環境を更新（値は既に評価済み）
                    for ((name, _), value) in bindings.iter().zip(new_values.iter()) {
                        loop_env_rc.write().set(name.clone(), value.clone());
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// quasiquoteを評価
    pub(super) fn eval_quasiquote(
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
                let mut result = Vec::with_capacity(1 + args.len());
                result.push(self.eval_quasiquote(func, Arc::clone(&env), depth)?);
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
                let mut result = Vec::with_capacity(4);
                result.push(Value::Symbol(crate::intern::intern_symbol("if")));
                result.push(self.eval_quasiquote(test, Arc::clone(&env), depth)?);
                result.push(self.eval_quasiquote(then, Arc::clone(&env), depth)?);
                if let Some(o) = otherwise {
                    result.push(self.eval_quasiquote(o, Arc::clone(&env), depth)?);
                }
                Ok(Value::List(result.into()))
            }
            Expr::Do { exprs, .. } => {
                let mut result = Vec::with_capacity(1 + exprs.len());
                result.push(Value::Symbol(crate::intern::intern_symbol("do")));
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
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("fn"))];
                let param_vals: Vec<Value> = if *is_variadic && params.len() == 1 {
                    vec![
                        Value::Symbol(crate::intern::intern_symbol("&")),
                        self.fn_param_to_value(&params[0]),
                    ]
                } else if *is_variadic {
                    let mut v: Vec<Value> = params[..params.len() - 1]
                        .iter()
                        .map(|p| self.fn_param_to_value(p))
                        .collect();
                    v.push(Value::Symbol(crate::intern::intern_symbol("&")));
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
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("let"))];
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
                    Value::Symbol(crate::intern::intern_symbol("def")),
                    Value::Symbol(crate::intern::intern_symbol(name)),
                ];
                items.push(self.eval_quasiquote(value, env, depth)?);
                Ok(Value::List(items.into()))
            }
            // その他は変換してValueに
            _ => self.expr_to_value(expr),
        }
    }

    /// ExprをValueに変換（データとして扱う）
    pub(super) fn expr_to_value(&self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Nil { .. } => Ok(Value::Nil),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::Integer { value, .. } => Ok(Value::Integer(*value)),
            Expr::Float { value, .. } => Ok(Value::Float(*value)),
            Expr::String { value, .. } => Ok(Value::String(value.clone())),
            Expr::Symbol { name, .. } => Ok(Value::Symbol(crate::intern::intern_symbol(name))),
            Expr::Keyword { name, .. } => Ok(Value::Keyword(crate::intern::intern_keyword(name))),
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
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("if"))];
                items.push(self.expr_to_value(test)?);
                items.push(self.expr_to_value(then)?);
                if let Some(o) = otherwise {
                    items.push(self.expr_to_value(o)?);
                }
                Ok(Value::List(items.into()))
            }
            Expr::Do { exprs, .. } => {
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("do"))];
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
                    Value::Symbol(crate::intern::intern_symbol("def")),
                    Value::Symbol(crate::intern::intern_symbol(name)),
                    self.expr_to_value(value)?,
                ]
                .into(),
            )),
            Expr::Let { bindings, body, .. } => {
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("let"))];
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
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("fn"))];
                let param_vals: Vec<Value> = if *is_variadic && params.len() == 1 {
                    vec![
                        Value::Symbol(crate::intern::intern_symbol("&")),
                        self.fn_param_to_value(&params[0]),
                    ]
                } else if *is_variadic {
                    let mut v: Vec<Value> = params[..params.len() - 1]
                        .iter()
                        .map(|p| self.fn_param_to_value(p))
                        .collect();
                    v.push(Value::Symbol(crate::intern::intern_symbol("&")));
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
                    Value::Symbol(crate::intern::intern_symbol("quasiquote")),
                    self.expr_to_value(e)?,
                ]
                .into(),
            )),
            Expr::Unquote { expr: e, .. } => Ok(Value::List(
                vec![Value::Symbol(crate::intern::intern_symbol("unquote")), self.expr_to_value(e)?].into(),
            )),
            Expr::UnquoteSplice { expr: e, .. } => Ok(Value::List(
                vec![
                    Value::Symbol(crate::intern::intern_symbol("unquote-splice")),
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
    pub(super) fn expand_macro(
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

    /// ValueをExprに変換（マクロ展開の結果をコードとして扱う、evalでも使用）
    pub fn value_to_expr(&self, val: &Value) -> Result<Expr, String> {
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
                    match &**s {
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
                                        if let Some(Value::String(desc)) = doc_map
                                            .get(&crate::value::MapKey::String("desc".to_string()))
                                        {
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
                                        doc_string = doc_map
                                            .get(&crate::value::MapKey::String("desc".to_string()))
                                            .and_then(|v| match v {
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
                                    let mut fn_items = vec![
                                        Value::Symbol(crate::intern::intern_symbol("fn")),
                                        params,
                                    ];
                                    fn_items.extend(body);
                                    let fn_value = Value::List(fn_items.into());

                                    // (def name (fn ...)) を構築
                                    let def_items = vec![
                                        Value::Symbol(crate::intern::intern_symbol("def")),
                                        Value::Symbol(crate::intern::intern_symbol(name)),
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
}

use super::helpers::qerr;
