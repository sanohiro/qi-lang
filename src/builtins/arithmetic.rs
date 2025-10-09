//! 算術演算関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

/// + - 加算
pub fn native_add(args: &[Value]) -> Result<Value, String> {
    let mut sum = 0;
    for arg in args {
        match arg {
            Value::Integer(n) => sum += n,
            _ => return Err(fmt_msg(MsgKey::TypeOnlyWithDebug, &["+", "integers", &format!("{:?}", arg)])),
        }
    }
    Ok(Value::Integer(sum))
}

/// - - 減算（または符号反転）
pub fn native_sub(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["-", "1"]));
    }

    match &args[0] {
        Value::Integer(first) => {
            if args.len() == 1 {
                Ok(Value::Integer(-first))
            } else {
                let mut result = *first;
                for arg in &args[1..] {
                    match arg {
                        Value::Integer(n) => result -= n,
                        _ => return Err(fmt_msg(MsgKey::TypeOnlyWithDebug, &["-", "integers", &format!("{:?}", arg)])),
                    }
                }
                Ok(Value::Integer(result))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnlyWithDebug, &["-", "integers", &format!("{:?}", args[0])])),
    }
}

/// * - 乗算
pub fn native_mul(args: &[Value]) -> Result<Value, String> {
    let mut product = 1;
    for arg in args {
        match arg {
            Value::Integer(n) => product *= n,
            _ => return Err(fmt_msg(MsgKey::TypeOnlyWithDebug, &["*", "integers", &format!("{:?}", arg)])),
        }
    }
    Ok(Value::Integer(product))
}

/// / - 除算
pub fn native_div(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["/"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Integer(a / b))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["/", "integers"])),
    }
}

/// % - 剰余
pub fn native_mod(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["%"]));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Integer(a % b))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["%", "integers"])),
    }
}

/// abs - 絶対値
pub fn native_abs(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["abs"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["abs", "numbers"])),
    }
}

/// min - 最小値
pub fn native_min(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["min", "1"]));
    }
    let mut min = match &args[0] {
        Value::Integer(n) => *n,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["min", "integers"])),
    };
    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if *n < min {
                    min = *n;
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["min", "integers"])),
        }
    }
    Ok(Value::Integer(min))
}

/// max - 最大値
pub fn native_max(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["max", "1"]));
    }
    let mut max = match &args[0] {
        Value::Integer(n) => *n,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["max", "integers"])),
    };
    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if *n > max {
                    max = *n;
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["max", "integers"])),
        }
    }
    Ok(Value::Integer(max))
}

/// inc - インクリメント
pub fn native_inc(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["inc"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n + 1)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["inc", "integers"])),
    }
}

/// dec - デクリメント
pub fn native_dec(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["dec"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n - 1)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["dec", "integers"])),
    }
}

/// sum - 合計
pub fn native_sum(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sum"]));
    }
    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let mut sum = 0;
            for item in items {
                match item {
                    Value::Integer(n) => sum += n,
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["sum", "integers"])),
                }
            }
            Ok(Value::Integer(sum))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sum", "lists or vectors"])),
    }
}
