//! Core文字列関数
//!
//! 文字列基本（3個）: str, split, join

use crate::i18n::{fmt_msg, msg, MsgKey};
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
        return Err(fmt_msg(MsgKey::Need2Args, &["split"]));
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
        return Err(fmt_msg(MsgKey::Need2Args, &["join"]));
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

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
pub const FUNCTIONS: super::NativeFunctions = &[
    ("str", native_str),
    ("split", native_split),
    ("join", native_join),
];
