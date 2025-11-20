//! 評価器のヘルパー関数
//!
//! 複数のモジュールで共有されるユーティリティ関数を提供します。
//!
//! - エラー処理のヘルパー
//! - 変数名の類似度検索
//! - ビルトイン関数の引数評価
//! - パターンからValueへの変換

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Env, Expr, Pattern, Value};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

use super::Evaluator;

// ========================================
// エラーヘルパー関数
// ========================================

/// エラーメッセージを生成
///
/// eval.rs内で統一的なエラーメッセージを生成するためのヘルパー。
/// i18n化されたメッセージを返す。
///
/// # 引数
/// * `key` - メッセージキー
/// * `args` - メッセージパラメータ
///
/// # 戻り値
/// フォーマット済みエラーメッセージ文字列
#[inline]
pub(super) fn qerr(key: MsgKey, args: &[&str]) -> String {
    fmt_msg(key, args)
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
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashSet};

    // BinaryHeap で上位 limit 件のみを保持（距離が小さい順、つまり類似度が高い順）
    let mut heap: BinaryHeap<Reverse<(usize, Arc<str>)>> = BinaryHeap::new();
    let mut seen = HashSet::new();

    // 親環境チェーンを辿って全階層から候補を収集
    let mut current = Some(env);
    while let Some(e) = current {
        for (name, _) in e.bindings() {
            // 既に処理済みの名前はスキップ（子環境が優先）
            if seen.contains(name) {
                continue;
            }
            seen.insert(name.clone());

            let distance = strsim::levenshtein(target, name);
            if distance <= max_distance {
                heap.push(Reverse((distance, name.clone())));
                // limit を超えたら最も類似度が低いものを削除
                if heap.len() > limit {
                    heap.pop();
                }
            }
        }
        // 親環境に移動
        current = e.parent().and_then(|p| {
            let guard = p.try_read();
            guard.as_ref().map(|g| unsafe {
                // SAFETY: この参照は次のループ開始前に破棄される
                std::mem::transmute::<&Env, &Env>(&**g)
            })
        });
    }

    // BinaryHeap から結果を取り出し（距離が小さい順）
    let mut results: Vec<_> = heap.into_iter().map(|Reverse((_, name))| name).collect();
    results.reverse(); // 距離が小さい順にソート
    results.into_iter().map(|s| s.to_string()).collect()
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
        let vals: SmallVec<[Value; 2]> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<SmallVec<_>, _>>()?;
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
        let vals: SmallVec<[Value; 3]> = args
            .iter()
            .map(|e| self.eval_with_env(e, Arc::clone(&env)))
            .collect::<Result<SmallVec<_>, _>>()?;
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
            Pattern::Var(name) => Value::Symbol(crate::intern::intern_symbol(name)),
            Pattern::List(params, rest) | Pattern::Vector(params, rest) => {
                let mut items: Vec<Value> =
                    params.iter().map(|p| self.fn_param_to_value(p)).collect();
                // restがある場合は [..., "&", rest_name] の形式にする
                if let Some(rest_param) = rest {
                    items.push(Value::Symbol(crate::intern::intern_symbol("&")));
                    items.push(self.fn_param_to_value(rest_param));
                }
                // Listパターンの場合はList、Vectorパターンの場合はVectorとして返す
                match param {
                    Pattern::List(_, _) => Value::List(items.into()),
                    _ => Value::Vector(items.into()),
                }
            }
            Pattern::Map(pairs, as_var) => {
                let mut map: HashMap<String, Value> = HashMap::new();
                for (key, pattern) in pairs {
                    map.insert(key.to_string(), self.fn_param_to_value(pattern));
                }
                // :as がある場合は追加
                if let Some(var) = as_var {
                    map.insert(
                        "as".to_string(),
                        Value::Symbol(crate::intern::intern_symbol(var.as_ref())),
                    );
                }
                Value::Map(map.into())
            }
            Pattern::As(inner, var) => {
                // (:as inner-pattern var) の形式で表現
                Value::List(
                    vec![
                        Value::Symbol(crate::intern::intern_symbol("as")),
                        self.fn_param_to_value(inner),
                        Value::Symbol(crate::intern::intern_symbol(var)),
                    ]
                    .into(),
                )
            }
            // match専用パターン（quote/マクロでも表現）
            Pattern::Wildcard => Value::Symbol(crate::intern::intern_symbol("_")),
            Pattern::Nil => Value::Nil,
            Pattern::Bool(b) => Value::Bool(*b),
            Pattern::Integer(n) => Value::Integer(*n),
            Pattern::Float(f) => Value::Float(*f),
            Pattern::String(s) => Value::String(s.clone()),
            Pattern::Keyword(k) => Value::Keyword(crate::intern::intern_keyword(k)),
            Pattern::Transform(var, expr) => {
                // (:transform var expr) の形式で表現
                Value::List(
                    vec![
                        Value::Symbol(crate::intern::intern_symbol("transform")),
                        Value::Symbol(crate::intern::intern_symbol(var)),
                        self.expr_to_value(expr).unwrap_or(Value::Nil),
                    ]
                    .into(),
                )
            }
            Pattern::Or(patterns) => {
                // (:or pattern1 pattern2 ...) の形式で表現
                let mut items = vec![Value::Symbol(crate::intern::intern_symbol("or"))];
                items.extend(patterns.iter().map(|p| self.fn_param_to_value(p)));
                Value::List(items.into())
            }
        }
    }
}
