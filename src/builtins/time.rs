//! 日時処理関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use chrono::{DateTime, TimeZone, Utc};

/// now-iso - 現在時刻をISO 8601形式で取得
pub fn native_now_iso(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["time/now-iso"]));
    }

    let now = Utc::now();
    Ok(Value::String(now.to_rfc3339()))
}

/// from-unix - Unixタイムスタンプ（秒）をISO 8601形式に変換
pub fn native_from_unix(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/from-unix"]));
    }

    match &args[0] {
        Value::Integer(timestamp) => {
            match Utc.timestamp_opt(*timestamp, 0) {
                chrono::LocalResult::Single(dt) => Ok(Value::String(dt.to_rfc3339())),
                _ => Err(fmt_msg(MsgKey::InvalidTimestamp, &["time/from-unix"])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/from-unix", "integers"])),
    }
}

/// to-unix - ISO 8601形式の文字列をUnixタイムスタンプ（秒）に変換
pub fn native_to_unix(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/to-unix"]));
    }

    match &args[0] {
        Value::String(s) => {
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => Ok(Value::Integer(dt.timestamp())),
                Err(_) => Err(fmt_msg(MsgKey::InvalidDateFormat, &["time/to-unix", s])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/to-unix", "strings"])),
    }
}

/// format - タイムスタンプを指定フォーマットで文字列化
/// 第1引数: Unixタイムスタンプ（整数）またはISO 8601文字列
/// 第2引数: フォーマット文字列（strftime形式）
pub fn native_format(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/format"]));
    }

    let dt = match &args[0] {
        Value::Integer(timestamp) => {
            match Utc.timestamp_opt(*timestamp, 0) {
                chrono::LocalResult::Single(dt) => dt,
                _ => return Err(fmt_msg(MsgKey::InvalidTimestamp, &["time/format"])),
            }
        }
        Value::String(s) => {
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => return Err(fmt_msg(MsgKey::InvalidDateFormat, &["time/format", s])),
            }
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["time/format (timestamp)", "integers or strings"])),
    };

    match &args[1] {
        Value::String(format_str) => {
            let formatted = dt.format(format_str).to_string();
            Ok(Value::String(formatted))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/format (format)", "strings"])),
    }
}
