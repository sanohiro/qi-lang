//! Core数値・比較演算関数
//!
//! 算術演算（11個）: +, -, *, /, %, inc, dec, abs, min, max, sum
//! 比較演算（6個）: =, !=, <, >, <=, >=
//! 合計17個のCore関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

// ========================================
// 算術演算（11個）
// ========================================

/// + - 加算
pub fn native_add(args: &[Value]) -> Result<Value, String> {
    let mut int_sum = 0i64;
    let mut float_sum = 0.0f64;
    let mut has_float = false;

    for arg in args {
        match arg {
            Value::Integer(n) => {
                if has_float {
                    float_sum += *n as f64;
                } else {
                    int_sum += n;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    float_sum = int_sum as f64;
                    has_float = true;
                }
                float_sum += f;
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnlyWithDebug,
                    &["+", "numbers", &format!("{:?}", arg)],
                ))
            }
        }
    }

    if has_float {
        Ok(Value::Float(float_sum))
    } else {
        Ok(Value::Integer(int_sum))
    }
}

/// - - 減算（または符号反転）
pub fn native_sub(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["-", "1"]));
    }

    // 符号反転（単項マイナス）
    if args.len() == 1 {
        return match &args[0] {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(fmt_msg(
                MsgKey::TypeOnlyWithDebug,
                &["-", "numbers", &format!("{:?}", args[0])],
            )),
        };
    }

    // 減算
    let mut has_float = false;
    let mut result_int = 0i64;
    let mut result_float = 0.0f64;

    // 最初の値を設定
    match &args[0] {
        Value::Integer(n) => result_int = *n,
        Value::Float(f) => {
            has_float = true;
            result_float = *f;
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnlyWithDebug,
                &["-", "numbers", &format!("{:?}", args[0])],
            ))
        }
    }

    // 残りの値を減算
    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if has_float {
                    result_float -= *n as f64;
                } else {
                    result_int -= n;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    result_float = result_int as f64;
                    has_float = true;
                }
                result_float -= f;
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnlyWithDebug,
                    &["-", "numbers", &format!("{:?}", arg)],
                ))
            }
        }
    }

    if has_float {
        Ok(Value::Float(result_float))
    } else {
        Ok(Value::Integer(result_int))
    }
}

/// * - 乗算
pub fn native_mul(args: &[Value]) -> Result<Value, String> {
    let mut int_product = 1i64;
    let mut float_product = 1.0f64;
    let mut has_float = false;

    for arg in args {
        match arg {
            Value::Integer(n) => {
                if has_float {
                    float_product *= *n as f64;
                } else {
                    int_product *= n;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    float_product = int_product as f64;
                    has_float = true;
                }
                float_product *= f;
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnlyWithDebug,
                    &["*", "numbers", &format!("{:?}", arg)],
                ))
            }
        }
    }

    if has_float {
        Ok(Value::Float(float_product))
    } else {
        Ok(Value::Integer(int_product))
    }
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
        (Value::Float(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float(a / b))
        }
        (Value::Integer(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float(*a as f64 / b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float(a / *b as f64))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["/", "numbers"])),
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

// ========================================
// 比較演算（6個）
// ========================================

/// 値の等価性を判定するヘルパー関数
/// ListとVectorは内容が同じなら等しいと見なす
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
        // ListとVectorは内容が同じなら等しい（Lisp系言語の一般的な仕様）
        (Value::List(a), Value::List(b))
        | (Value::Vector(a), Value::Vector(b))
        | (Value::List(a), Value::Vector(b))
        | (Value::Vector(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Map(a), Value::Map(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| values_equal(v, bv)))
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
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a < (*b as f64))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["<", "numbers"])),
    }
}

/// > - 大なり比較
pub fn native_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &[">"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a > (*b as f64))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[">", "numbers"])),
    }
}

/// <= - 小なりイコール比較
pub fn native_le(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["<="]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a <= (*b as f64))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["<=", "numbers"])),
    }
}

/// >= - 大なりイコール比較
pub fn native_ge(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &[">="]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a >= (*b as f64))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[">=", "numbers"])),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
pub const FUNCTIONS: super::NativeFunctions = &[
    ("+", native_add),
    ("-", native_sub),
    ("*", native_mul),
    ("/", native_div),
    ("%", native_mod),
    ("abs", native_abs),
    ("min", native_min),
    ("max", native_max),
    ("inc", native_inc),
    ("dec", native_dec),
    ("sum", native_sum),
    ("=", native_eq),
    ("!=", native_ne),
    ("<", native_lt),
    (">", native_gt),
    ("<=", native_le),
    (">=", native_ge),
];
