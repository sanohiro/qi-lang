//! パターンマッチング処理
//!
//! match式の評価とパターンマッチングロジックを提供します。

use crate::builtins::util::to_map_key;
use crate::i18n::{msg, MsgKey};
use crate::value::{Env, Expr, MatchArm, Pattern, Value};
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::sync::Arc;

use super::Evaluator;

/// 通常のパターンマッチで使用される変数束縛の数
/// ヒープ割り当てを回避するための最適化（多くのパターンは8変数以下）
const TYPICAL_PATTERN_BINDINGS: usize = 8;

impl Evaluator {
    /// match式を評価
    ///
    /// 各アーム（パターン + ガード + ボディ）を順番に評価し、
    /// 最初にマッチしたアームの本体を実行して結果を返す。
    ///
    /// # 引数
    /// - `value`: マッチ対象の値
    /// - `arms`: マッチアームのリスト（パターン、ガード、ボディ）
    /// - `env`: 評価環境
    ///
    /// # エラー
    /// - パターンマッチング中のエラー
    /// - ガード条件評価中のエラー
    /// - マッチしたアームのボディ評価中のエラー
    /// - すべてのパターンがマッチしなかった場合
    pub(super) fn eval_match(
        &self,
        value: &Value,
        arms: &[MatchArm],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        for arm in arms {
            let mut bindings: SmallVec<[(String, Value); TYPICAL_PATTERN_BINDINGS]> =
                SmallVec::new();
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
                        guard_env.set(name.as_str(), val.clone());
                    }
                    let guard_val = self.eval_with_env(guard, Arc::new(RwLock::new(guard_env)))?;
                    if !guard_val.is_truthy() {
                        continue;
                    }
                }

                // 変換を適用
                let mut match_env = Env::with_parent(Arc::clone(&env));
                for (name, val) in bindings {
                    match_env.set(name.as_str(), val.clone());
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

    /// 変換パターンを含むパターンマッチング
    ///
    /// 通常のパターンマッチングに加えて、Transform パターン（`x :as (transform ...)`）
    /// を処理する。変換情報は後で適用するために記録される。
    ///
    /// # 引数
    /// - `pattern`: マッチさせるパターン
    /// - `value`: マッチ対象の値
    /// - `bindings`: 変数束縛を記録する可変参照
    /// - `transforms`: 変換情報（変数名、変換式、元の値）を記録する可変参照
    ///
    /// Transformパターン（`:as (transform expr)`）を含むパターンマッチング
    ///
    /// このメソッドは通常の`match_pattern`と異なり、Transformパターンのネストに対応します。
    ///
    /// # Transformパターンのネスト対応
    ///
    /// 例: `[x :as inc, [y z :as (* 2)]]`
    ///   → Vector内にTransformパターンとネストしたVectorパターンが混在
    ///
    /// 処理フロー:
    /// 1. Vector/List/Map/Asパターンは再帰的にこのメソッドを呼び出す
    /// 2. Transformパターンは変換情報を`transforms`に記録（後で`apply_transform`で適用）
    /// 3. その他のパターン（Wildcard/Literal/Var/Or）は通常の`match_pattern`で処理
    ///
    /// # なぜ変換を後で適用するのか
    ///
    /// パターンマッチング時には元の値を保持する必要があります。
    /// ガード評価後に変換を適用することで、ガード内で元の値を参照可能にします。
    ///
    /// 例:
    /// ```qi
    /// (match 10
    ///   x :as inc (if (> x 5) x)  ; ガード内ではxは10（変換前）
    ///   _ "default")
    /// ; → マッチ成功後、xに変換（inc 10 = 11）を適用
    /// ```
    ///
    /// # 引数
    /// - `bindings`: 変数束縛を記録（SmallVec: 通常8個以下でヒープ割り当て回避）
    /// - `transforms`: 変換情報を記録（変数名、変換式、元の値）
    ///
    /// # 戻り値
    /// - `Ok(true)`: マッチ成功
    /// - `Ok(false)`: マッチ失敗
    /// - `Err`: マッチング中のエラー
    fn match_pattern_with_transforms(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut SmallVec<[(String, Value); TYPICAL_PATTERN_BINDINGS]>,
        transforms: &mut Vec<(String, Expr, Value)>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Transform(var, transform) => {
                // 変換情報を記録（後でapply_transformで適用）
                transforms.push((var.to_string(), (**transform).clone(), value.clone()));
                // bindingsには元の値を設定（ガード評価で使用）
                bindings.push((var.to_string(), value.clone()));
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
                    // im::Vector に直接 collect（Vec → im::Vector の二重アロケーション回避）
                    let rest_values: im::Vector<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern_with_transforms(
                        rest_pattern,
                        &Value::Vector(rest_values),
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
                    // im::Vector に直接 collect（Vec → im::Vector の二重アロケーション回避）
                    let rest_values: im::Vector<Value> =
                        values.iter().skip(patterns.len()).cloned().collect();
                    self.match_pattern_with_transforms(
                        rest_pattern,
                        &Value::List(rest_values),
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
                        bindings.push((var.to_string(), value.clone()));
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::As(inner_pattern, var) => {
                // ネストされたTransformパターンを扱うため、再帰的に処理
                if self.match_pattern_with_transforms(inner_pattern, value, bindings, transforms)? {
                    bindings.push((var.to_string(), value.clone()));
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

    /// 変換式を値に適用
    ///
    /// パターンマッチング時に記録された変換式を評価し、値に適用する。
    /// 変換式は関数またはシンボルとして評価され、(transform value) の形で呼び出される。
    ///
    /// # 引数
    /// - `transform`: 変換式
    /// - `value`: 変換対象の値
    /// - `env`: 評価環境
    ///
    /// # 戻り値
    /// 変換後の値
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

    /// パターンマッチング（変換なし）
    ///
    /// 標準的なパターンマッチングを実行する。以下のパターンをサポート:
    /// - ワイルドカード (_)
    /// - リテラル (nil, bool, integer, float, string, keyword)
    /// - 変数 (x)
    /// - ベクトル/リスト ([x y z], (x y z), [x y & rest])
    /// - マップ ({:key value :as m})
    /// - as パターン (pattern :as var)
    /// - or パターン (pattern1 | pattern2)
    ///
    /// # 引数
    /// - `pattern`: マッチさせるパターン
    /// - `value`: マッチ対象の値
    /// - `bindings`: 変数束縛を記録する可変参照
    ///
    /// # 戻り値
    /// - `Ok(true)`: マッチ成功
    /// - `Ok(false)`: マッチ失敗
    /// - `Err`: マッチング中のエラー
    #[allow(clippy::only_used_in_recursion)]
    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut SmallVec<[(String, Value); TYPICAL_PATTERN_BINDINGS]>,
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
            Pattern::Keyword(k) => Ok(matches!(value, Value::Keyword(vk) if **vk == **k)),
            Pattern::Var(name) => {
                bindings.push((name.to_string(), value.clone()));
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
                        bindings.push((var.to_string(), value.clone()));
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
                    bindings.push((var.to_string(), value.clone()));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Or(patterns) => {
                // Orパターンのバックトラッキング（効率的なロールバック実装）
                //
                // 例: (x | [y z])
                //   → 最初のパターン `x` がマッチ失敗したら、`[y z]` を試す
                //
                // ロールバック戦略:
                //   各パターン試行前にbindingsの長さを記録し、
                //   失敗時にtruncateでロールバック（cloneより高速）
                //
                // 注意: 最初にマッチしたパターンを採用（短絡評価）
                for pat in patterns {
                    let start_len = bindings.len();
                    if self.match_pattern(pat, value, bindings)? {
                        // 最初にマッチしたパターンを使う
                        return Ok(true);
                    }
                    // 失敗時はbindingsを元の長さにロールバック
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
}
