//! 比較演算関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// = - 等価比較
pub fn native_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["="]));
    }
    Ok(Value::Bool(args[0] == args[1]))
}

/// != - 非等価比較
pub fn native_ne(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["!="]));
    }
    Ok(Value::Bool(args[0] != args[1]))
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
