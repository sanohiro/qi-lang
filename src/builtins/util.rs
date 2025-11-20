//! ユーティリティ関数（デバッグ、Railway Pipeline、Result型ヘルパー）

use crate::check_args;
use crate::value::{MapKey, Value};

/// キーワード形式のマップキーを生成
///
/// 文字列からキーワードキー（`:key`形式）を生成する
pub fn to_map_key(key: &str) -> crate::value::MapKey {
    crate::value::MapKey::Keyword(crate::intern::intern_keyword(key))
}

/// HashMap<String, Value>をHashMap<MapKey, Value>に変換
///
/// 文字列キーをMapKey::Stringに変換する
pub fn convert_string_map_to_mapkey(
    map: std::collections::HashMap<String, Value>,
) -> crate::HashMap<MapKey, Value> {
    map.into_iter()
        .map(|(k, v)| (MapKey::String(k), v))
        .collect()
}

// ========================================
// Railway Pipeline関数
// ========================================

/// _railway-pipe - Railway Oriented Programming用の内部関数
///
/// **仕様: {:error}以外は全て成功**
///
/// 入力値の処理:
/// - {:error ...} → ショートサーキット（関数を実行しない）
/// - その他 → そのまま関数に渡す
///
/// 出力値の処理:
/// - {:error ...} → そのまま返す（エラー伝播）
/// - その他 → そのまま返す（値そのまま）
///
/// # 例
/// ```ignore
/// (10 |>? (fn [x] (* x 2)))  ;; => 20
/// (10 |>? (fn [x] {:error "fail"}))  ;; => {:error "fail"}
/// ```
///
/// # Railway Oriented Programming (ROP) の実装
///
/// ## 概念
/// - 成功パス（正常な値）とエラーパス（`{:error ...}`）を分離
/// - エラー発生時は後続の関数をスキップ（ショートサーキット）
/// - `Result<T, E>`を使わずに値の中でエラーを表現
///
/// ## 利点
/// - パイプライン内でエラーハンドリングを自然に記述
/// - 例: `(parse-json data) |>? (get :user) |>? (validate)`
///   → どこかでエラーが発生したら後続をスキップ
///
/// ## 設計
/// - `{:error ...}`のみをエラーとして扱う（シンプルで一貫性がある）
/// - その他の全ての値（`nil`含む）は成功として扱う
/// - エラーマップはそのまま返す（ラップしない）
///
/// ## なぜ`{:error}`だけをエラーとするのか
/// - `nil`は有効な値として扱いたいケースがある（例: オプショナル値）
/// - 明示的なエラー表現により、意図が明確になる
/// - `{:error "message" :code 404}`のような詳細なエラー情報を含められる
pub fn native_railway_pipe(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    check_args!(args, 2, "_railway-pipe");

    let func = &args[0];
    let input = &args[1];

    // 入力値の処理
    let value_to_pass = match input {
        Value::Map(m) => {
            // {:error ...}ならショートサーキット（後続の関数をスキップ）
            if m.contains_key(&crate::constants::keywords::error_mapkey()) {
                return Ok(input.clone());
            }
            // その他のマップはそのまま渡す
            else {
                input
            }
        }
        // マップ以外（nil含む）はそのまま渡す
        _ => input,
    };

    // 関数を実行して結果をそのまま返す
    // {:error}以外は全て成功なので、ラップしない
    evaluator.apply_function(func, std::slice::from_ref(value_to_pass))
}

/// inspect - 値を整形して表示（デバッグ用）
pub fn native_inspect(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "inspect");

    println!("{}", pretty_print(&args[0], 0));
    Ok(args[0].clone())
}

/// 値を整形して表示するヘルパー関数
fn pretty_print(value: &Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match value {
        Value::Map(m) => {
            if m.is_empty() {
                return "{}".to_string();
            }
            let mut lines = vec!["{".to_string()];
            for (k, v) in m {
                lines.push(format!(
                    "{}  \"{}\": {}",
                    indent_str,
                    k,
                    pretty_print(v, indent + 1)
                ));
            }
            lines.push(format!("{}}}", indent_str));
            lines.join("\n")
        }
        Value::Vector(v) => {
            if v.is_empty() {
                return "[]".to_string();
            }
            let mut lines = vec!["[".to_string()];
            for item in v {
                lines.push(format!(
                    "{}  {}",
                    indent_str,
                    pretty_print(item, indent + 1)
                ));
            }
            lines.push(format!("{}]", indent_str));
            lines.join("\n")
        }
        Value::List(l) => {
            if l.is_empty() {
                return "()".to_string();
            }
            let mut lines = vec!["(".to_string()];
            for item in l {
                lines.push(format!(
                    "{}  {}",
                    indent_str,
                    pretty_print(item, indent + 1)
                ));
            }
            lines.push(format!("{})", indent_str));
            lines.join("\n")
        }
        Value::String(s) => format!("\"{}\"", s),
        _ => value.to_string(),
    }
}

/// time - 関数実行時間を計測
pub fn native_time(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    check_args!(args, 1, "time");

    let start = std::time::Instant::now();
    let result = evaluator.apply_function(&args[0], &[])?;
    let elapsed = start.elapsed();

    println!("Elapsed: {:.3}s", elapsed.as_secs_f64());
    Ok(result)
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category util
/// @qi-doc:functions inspect
///
/// 注意:
/// - _railway-pipe, timeはEvaluatorが必要なため、mod.rsで別途登録されます
/// - commentは特殊形式として実装されています（eval/call.rsで処理）
pub const FUNCTIONS: super::NativeFunctions = &[("inspect", native_inspect)];
