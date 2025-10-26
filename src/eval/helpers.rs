//! 評価器のヘルパー関数
//!
//! 複数のモジュールで共有されるユーティリティ関数を提供します。
//!
//! - エラー処理のヘルパー
//! - 変数名の類似度検索
//! - ビルトイン関数の引数評価
//! - パターンからValueへの変換

use crate::error::QiError;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Env, Expr, Pattern, Value};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::Evaluator;

// ========================================
// エラーヘルパー関数
// ========================================

/// QiErrorを使ってエラーメッセージを生成
///
/// eval.rs内で統一的なエラーメッセージを生成するためのヘルパー。
/// エラーコードを自動的に付与し、i18n化されたメッセージを返す。
///
/// # 引数
/// * `key` - メッセージキー
/// * `args` - メッセージパラメータ
///
/// # 戻り値
/// フォーマット済みエラーメッセージ文字列
#[inline]
pub(super) fn qerr(key: MsgKey, args: &[&str]) -> String {
    let base_msg = fmt_msg(key, args);
    let error_code = QiError::error_code_from_eval_msg(&key);
    QiError::new(error_code, base_msg).into()
}

// ========================================
// 変数名候補検索
// ========================================

/// 環境から変数名の候補を取得
///
/// 未定義変数エラーの際に、類似した名前の変数を提案するためのヘルパー。
/// レーベンシュタイン距離を使って類似度を計算する。
///
/// # 引数
/// * `env` - 検索対象の環境
/// * `target` - 検索する変数名
/// * `max_distance` - 最大編集距離（通常は3）
/// * `limit` - 返す候補の最大数（通常は3）
///
/// # 戻り値
/// 類似度が高い順にソートされた変数名のリスト
pub(super) fn find_similar_names(
    env: &Env,
    target: &str,
    max_distance: usize,
    limit: usize,
) -> Vec<String> {
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

// ========================================
// ビルトイン関数評価ヘルパー
// ========================================

impl Evaluator {
    /// 2引数のbuiltin関数を評価する共通ヘルパー
    ///
    /// 引数の数をチェックし、各引数を評価してからビルトイン関数を呼び出す。
    ///
    /// # 引数
    /// * `args` - 評価前の引数式のスライス
    /// * `env` - 評価環境
    /// * `func_name` - 関数名（エラーメッセージ用）
    /// * `builtin` - 呼び出すビルトイン関数
    ///
    /// # エラー
    /// * 引数が2個でない場合
    /// * 引数の評価に失敗した場合
    /// * ビルトイン関数がエラーを返した場合
    #[inline]
    pub(super) fn eval_builtin_2_args(
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
    ///
    /// 引数の数をチェックし、各引数を評価してからビルトイン関数を呼び出す。
    ///
    /// # 引数
    /// * `args` - 評価前の引数式のスライス
    /// * `env` - 評価環境
    /// * `func_name` - 関数名（エラーメッセージ用）
    /// * `builtin` - 呼び出すビルトイン関数
    ///
    /// # エラー
    /// * 引数が3個でない場合
    /// * 引数の評価に失敗した場合
    /// * ビルトイン関数がエラーを返した場合
    #[inline]
    pub(super) fn eval_builtin_3_args(
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

    /// PatternをValueに変換（マクロ展開/quote用）
    ///
    /// 関数パラメータやletバインディングのパターンを、
    /// quoteやマクロ展開時にValueとして表現するための変換関数。
    ///
    /// # 変換例
    /// * `Pattern::Var("x")` → `Value::Symbol("x")`
    /// * `Pattern::Vector([a, b], Some(rest))` → `Value::Vector([a, b, "&", rest])`
    /// * `Pattern::Map({"x" => p}, Some("m"))` → `Value::Map({"x" => p, "as" => "m"})`
    ///
    /// # 引数
    /// * `param` - 変換するパターン
    ///
    /// # 戻り値
    /// パターンをValueとして表現したもの
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn fn_param_to_value(&self, param: &Pattern) -> Value {
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
}
