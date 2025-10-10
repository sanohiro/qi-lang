//! 数学関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use rand::Rng;

/// pow - べき乗
pub fn native_pow(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("pow requires 2 arguments (base, exponent)".to_string());
    }

    let base_f = match &args[0] {
        Value::Integer(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pow (base)", "numbers"])),
    };
    let exp_f = match &args[1] {
        Value::Integer(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pow (exponent)", "numbers"])),
    };

    // 両方が整数で指数が非負の場合は整数で返す
    if let (Value::Integer(base), Value::Integer(exp)) = (&args[0], &args[1]) {
        if *exp >= 0 {
            if let Some(result) = base.checked_pow(*exp as u32) {
                return Ok(Value::Integer(result));
            }
        }
    }

    Ok(Value::Float(base_f.powf(exp_f)))
}

/// sqrt - 平方根
pub fn native_sqrt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("sqrt requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sqrt", "numbers"])),
    }
}

/// round - 四捨五入
pub fn native_round(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("round requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.round() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["round", "numbers"])),
    }
}

/// floor - 切り捨て
pub fn native_floor(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("floor requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.floor() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["floor", "numbers"])),
    }
}

/// ceil - 切り上げ
pub fn native_ceil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("ceil requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.ceil() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["ceil", "numbers"])),
    }
}

/// clamp - 値を範囲内に制限
pub fn native_clamp(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("clamp requires 3 arguments (value, min, max)".to_string());
    }

    match (&args[0], &args[1], &args[2]) {
        (Value::Integer(val), Value::Integer(min), Value::Integer(max)) => {
            Ok(Value::Integer((*val).clamp(*min, *max)))
        }
        _ => {
            // floatの場合も対応
            let val = match &args[0] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (value)", "numbers"])),
            };
            let min = match &args[1] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (min)", "numbers"])),
            };
            let max = match &args[2] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (max)", "numbers"])),
            };
            Ok(Value::Float(val.clamp(min, max)))
        }
    }
}

/// rand - 0.0以上1.0未満の乱数
pub fn native_rand(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("rand takes no arguments".to_string());
    }

    let mut rng = rand::rng();
    Ok(Value::Float(rng.random::<f64>()))
}

/// rand-int - 0以上n未満の整数乱数
pub fn native_rand_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("rand-int requires 1 argument (upper bound)".to_string());
    }

    match &args[0] {
        Value::Integer(n) => {
            if *n <= 0 {
                return Err("rand-int: upper bound must be positive".to_string());
            }
            let mut rng = rand::rng();
            Ok(Value::Integer(rng.random_range(0..*n)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["rand-int", "integers"])),
    }
}
