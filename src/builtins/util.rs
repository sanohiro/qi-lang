//! ユーティリティ関数（デバッグ、Railway Pipeline、Result型ヘルパー）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

// ========================================
// Result型マップ生成ヘルパー（後方互換性のため残存）
// ========================================

/// {:error message} 形式のマップを生成
///
/// **非推奨**: 新しいコードでは `Value::error(message)` を使用してください
#[deprecated(note = "Use Value::error(message) instead")]
pub fn err_map(message: String) -> Value {
    Value::error(message)
}

/// キーワード形式のマップキーを生成
///
/// 文字列からキーワードキー（`:key`形式）を生成する
pub fn to_map_key(key: &str) -> String {
    format!(":{}", key)
}

// ========================================
// Railway Pipeline関数
// ========================================

/// _railway-pipe - Railway Oriented Programming用の内部関数
///
/// **新仕様: {:error}以外は全て成功**
///
/// 入力値の処理:
/// - {:error ...} → ショートサーキット（関数を実行しない）
/// - {:ok value} → valueを取り出して関数に渡す（後方互換性）
/// - その他 → そのまま関数に渡す
///
/// 出力値の処理:
/// - {:error ...} → そのまま返す（エラー伝播）
/// - その他 → そのまま返す（値そのまま！:okラップなし）
///
/// # 例
/// ```ignore
/// (10 |>? (fn [x] (* x 2)))  ;; => 20
/// (10 |>? (fn [x] {:error "fail"}))  ;; => {:error "fail"}
/// ```
pub fn native_railway_pipe(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["_railway-pipe"]));
    }

    let func = &args[0];
    let input = &args[1];

    // 入力値の処理
    let value_to_pass = match input {
        Value::Map(m) => {
            // {:error ...}ならショートサーキット
            if m.contains_key(":error") {
                return Ok(input.clone());
            }
            // {:ok value}なら値を取り出す（後方互換性）
            else if let Some(ok_val) = m.get(":ok") {
                ok_val
            }
            // その他のマップはそのまま渡す
            else {
                input
            }
        }
        // マップ以外はそのまま渡す
        _ => input,
    };

    // 関数を実行して結果をそのまま返す
    // {:error}以外は全て成功なので、ラップしない
    evaluator.apply_function(func, std::slice::from_ref(value_to_pass))
}

/// inspect - 値を整形して表示（デバッグ用）
pub fn native_inspect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["inspect"]));
    }

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
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time"]));
    }

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
/// 注意: _railway-pipe, timeはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[("inspect", native_inspect)];
