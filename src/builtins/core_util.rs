//! Coreユーティリティ関数
//!
//! 型変換（3個）: to-int, to-float, to-string
//! 日時（3個）: now, timestamp, sleep
//! 合計6個のCore関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ========================================
// 型変換（3個）
// ========================================

/// to-int - 値を整数に変換
pub fn native_to_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-int"]));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(*i)),
        Value::Float(f) => Ok(Value::Integer(*f as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| fmt_msg(MsgKey::CannotParseAsInt, &["to-int", s])),
        Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        _ => Err(fmt_msg(
            MsgKey::CannotConvertToInt,
            &["to-int", &format!("{:?}", args[0])],
        )),
    }
}

/// to-float - 値を浮動小数点数に変換
pub fn native_to_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-float"]));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| fmt_msg(MsgKey::CannotParseAsFloat, &["to-float", s])),
        _ => Err(fmt_msg(
            MsgKey::CannotConvertToFloat,
            &["to-float", &format!("{:?}", args[0])],
        )),
    }
}

/// to-string - 値を文字列に変換
pub fn native_to_string(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-string"]));
    }

    let s = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Keyword(k) => k.clone(),
        Value::Nil => "nil".to_string(),
        _ => format!("{:?}", args[0]),
    };

    Ok(Value::String(s))
}

// ========================================
// 日時（3個）
// ========================================

/// now - 現在時刻（UNIXタイムスタンプ秒）
pub fn native_now(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["now"]));
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("now: system time error: {}", e))?;

    Ok(Value::Integer(duration.as_secs() as i64))
}

/// timestamp - 現在時刻（UNIXタイムスタンプミリ秒）
pub fn native_timestamp(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["timestamp"]));
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("timestamp: system time error: {}", e))?;

    Ok(Value::Integer(duration.as_millis() as i64))
}

/// sleep - 指定ミリ秒スリープ
pub fn native_sleep(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["sleep", "1", "(milliseconds)"],
        ));
    }

    let millis = match &args[0] {
        Value::Integer(n) => {
            if *n < 0 {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["sleep", "duration"]));
            }
            *n as u64
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["sleep", "argument"])),
    };

    thread::sleep(Duration::from_millis(millis));
    Ok(Value::Nil)
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
pub const FUNCTIONS: super::NativeFunctions = &[
    ("to-int", native_to_int),
    ("to-float", native_to_float),
    ("to-string", native_to_string),
    ("now", native_now),
    ("timestamp", native_timestamp),
    ("sleep", native_sleep),
];
