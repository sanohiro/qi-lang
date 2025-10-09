//! 述語関数（型判定など）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// empty? - コレクションが空かどうか判定
pub fn native_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["empty?"]));
    }
    match &args[0] {
        Value::Nil => Ok(Value::Bool(true)),
        Value::List(v) | Value::Vector(v) => Ok(Value::Bool(v.is_empty())),
        Value::Map(m) => Ok(Value::Bool(m.is_empty())),
        Value::String(s) => Ok(Value::Bool(s.is_empty())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["empty?", "strings or collections"])),
    }
}

/// nil? - nilかどうか判定
pub fn native_nil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["nil?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

/// list? - リストかどうか判定
pub fn native_list_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["list?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

/// vector? - ベクタかどうか判定
pub fn native_vector_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["vector?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Vector(_))))
}

/// map? - マップかどうか判定
pub fn native_map_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["map?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Map(_))))
}

/// string? - 文字列かどうか判定
pub fn native_string_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["string?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// integer? - 整数かどうか判定
pub fn native_integer_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["integer?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Integer(_))))
}

/// float? - 浮動小数点数かどうか判定
pub fn native_float_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["float?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Float(_))))
}

/// keyword? - キーワードかどうか判定
pub fn native_keyword_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["keyword?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Keyword(_))))
}
