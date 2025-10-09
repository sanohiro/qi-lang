//! 比較演算関数

use crate::value::Value;

/// = - 等価比較
pub fn native_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("= には2つの引数が必要です".to_string());
    }
    Ok(Value::Bool(args[0] == args[1]))
}

/// != - 非等価比較
pub fn native_ne(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("!= には2つの引数が必要です".to_string());
    }
    Ok(Value::Bool(args[0] != args[1]))
}

/// < - 小なり比較
pub fn native_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("< には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
        _ => Err("< は整数のみ受け付けます".to_string()),
    }
}

/// > - 大なり比較
pub fn native_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("> には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
        _ => Err("> は整数のみ受け付けます".to_string()),
    }
}

/// <= - 小なりイコール比較
pub fn native_le(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("<= には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        _ => Err("<= は整数のみ受け付けます".to_string()),
    }
}

/// >= - 大なりイコール比較
pub fn native_ge(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(">= には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
        _ => Err(">= は整数のみ受け付けます".to_string()),
    }
}
