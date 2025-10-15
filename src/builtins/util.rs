//! ユーティリティ関数（デバッグ、Railway Pipeline）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// _railway-pipe - Railway Oriented Programming用の内部関数
///
/// {:ok value}なら関数に渡し、{:error e}ならそのまま返す
pub fn native_railway_pipe(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["_railway-pipe"]));
    }

    let func = &args[0];
    let result = &args[1];

    // resultが{:ok value}または{:error ...}の形式かチェック
    match result {
        Value::Map(m) => {
            // {:ok value}の場合は値を取り出して関数に渡す
            if let Some(ok_val) = m.get("ok") {
                evaluator.apply_function(func, &[ok_val.clone()])
            }
            // {:error e}の場合はそのまま返す(ショートサーキット)
            else if m.contains_key("error") {
                Ok(result.clone())
            } else {
                Err(fmt_msg(MsgKey::RailwayRequiresOkError, &[]))
            }
        }
        _ => Err(fmt_msg(MsgKey::RailwayRequiresOkError, &[])),
    }
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
