//! 比較演算関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// 値の等価性を判定するヘルパー関数
fn values_equal(a: &Value, b: &Value) -> bool {
    use std::ptr;
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::Keyword(a), Value::Keyword(b)) => a == b,
        (Value::List(a), Value::List(b)) | (Value::Vector(a), Value::Vector(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Map(a), Value::Map(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).map_or(false, |bv| values_equal(v, bv)))
        }
        (Value::Function(a), Value::Function(b)) => ptr::eq(&**a, &**b),
        (Value::NativeFunc(a), Value::NativeFunc(b)) => a.name == b.name,
        (Value::Macro(a), Value::Macro(b)) => ptr::eq(&**a, &**b),
        (Value::Atom(a), Value::Atom(b)) => ptr::eq(&**a, &**b),
        (Value::Channel(a), Value::Channel(b)) => ptr::eq(&**a, &**b),
        (Value::Uvar(a), Value::Uvar(b)) => a == b,
        _ => false,
    }
}

/// = - 等価比較
pub fn native_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["="]));
    }
    Ok(Value::Bool(values_equal(&args[0], &args[1])))
}

/// != - 非等価比較
pub fn native_ne(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["!="]));
    }
    Ok(Value::Bool(!values_equal(&args[0], &args[1])))
}

/// < - 小なり比較
pub fn native_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["<"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["<", "integers"])),
    }
}

/// > - 大なり比較
pub fn native_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &[">"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[">", "integers"])),
    }
}

/// <= - 小なりイコール比較
pub fn native_le(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["<="]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["<=", "integers"])),
    }
}

/// >= - 大なりイコール比較
pub fn native_ge(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &[">="]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[">=", "integers"])),
    }
}
