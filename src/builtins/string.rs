//! 文字列操作関数

use crate::i18n::{msg, MsgKey};
use crate::value::Value;

/// str - 値を文字列に変換して連結
pub fn native_str(args: &[Value]) -> Result<Value, String> {
    let s = args
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => format!("{}", v),
        })
        .collect::<String>();
    Ok(Value::String(s))
}

/// split - 文字列を分割
pub fn native_split(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("splitには2つの引数が必要です: 実際 {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            let parts: Vec<Value> = s
                .split(sep.as_str())
                .map(|p| Value::String(p.to_string()))
                .collect();
            Ok(Value::Vector(parts))
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// join - リストを文字列に結合
pub fn native_join(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("joinには2つの引数が必要です: 実際 {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(sep), Value::List(items)) | (Value::String(sep), Value::Vector(items)) => {
            let strings: Result<Vec<String>, String> = items
                .iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Ok(format!("{}", v)),
                })
                .collect();
            Ok(Value::String(strings?.join(sep)))
        }
        _ => Err(msg(MsgKey::JoinStringAndList).to_string()),
    }
}

/// upper - 文字列を大文字に変換
pub fn native_upper(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("upperには1つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(msg(MsgKey::UpperStringOnly).to_string()),
    }
}

/// lower - 文字列を小文字に変換
pub fn native_lower(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("lowerには1つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(msg(MsgKey::LowerStringOnly).to_string()),
    }
}

/// trim - 文字列の前後の空白を削除
pub fn native_trim(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("trimには1つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err(msg(MsgKey::TrimStringOnly).to_string()),
    }
}
