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
                    int_sum = int_sum
                        .checked_add(*n)
                        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &["+"]))?;
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
            Value::Integer(n) => {
                // ⚠️ SAFETY: i64::MIN の符号反転はオーバーフローするため checked_neg() を使用
                n.checked_neg()
                    .map(Value::Integer)
                    .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &["unary negation"]))
            }
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
                    result_int = result_int
                        .checked_sub(*n)
                        .ok_or_else(|| fmt_msg(MsgKey::IntegerUnderflow, &["-"]))?;
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
                    int_product = int_product
                        .checked_mul(*n)
                        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &["*"]))?;
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
/// 整数・浮動小数の両方に対応。浮動小数が1つでも含まれる場合は浮動小数で返す
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
        (Value::Integer(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float((*a as f64).rem_euclid(*b)))
        }
        (Value::Float(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float(a.rem_euclid(*b as f64)))
        }
        (Value::Float(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(msg(MsgKey::DivisionByZero).to_string());
            }
            Ok(Value::Float(a.rem_euclid(*b)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["%", "numbers"])),
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
/// 整数・浮動小数の両方に対応。浮動小数が1つでも含まれる場合は浮動小数で返す
pub fn native_min(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["min", "1"]));
    }

    let mut has_float = false;
    let mut min_int = i64::MAX;
    let mut min_float = f64::INFINITY;

    // 最初の値を設定
    match &args[0] {
        Value::Integer(n) => min_int = *n,
        Value::Float(f) => {
            has_float = true;
            min_float = *f;
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["min", "numbers"])),
    }

    // 残りの値と比較
    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if has_float {
                    if (*n as f64) < min_float {
                        min_float = *n as f64;
                    }
                } else if *n < min_int {
                    min_int = *n;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    min_float = min_int as f64;
                    has_float = true;
                }
                if *f < min_float {
                    min_float = *f;
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["min", "numbers"])),
        }
    }

    if has_float {
        Ok(Value::Float(min_float))
    } else {
        Ok(Value::Integer(min_int))
    }
}

/// max - 最大値
/// 整数・浮動小数の両方に対応。浮動小数が1つでも含まれる場合は浮動小数で返す
pub fn native_max(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["max", "1"]));
    }

    let mut has_float = false;
    let mut max_int = i64::MIN;
    let mut max_float = f64::NEG_INFINITY;

    // 最初の値を設定
    match &args[0] {
        Value::Integer(n) => max_int = *n,
        Value::Float(f) => {
            has_float = true;
            max_float = *f;
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["max", "numbers"])),
    }

    // 残りの値と比較
    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if has_float {
                    if (*n as f64) > max_float {
                        max_float = *n as f64;
                    }
                } else if *n > max_int {
                    max_int = *n;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    max_float = max_int as f64;
                    has_float = true;
                }
                if *f > max_float {
                    max_float = *f;
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["max", "numbers"])),
        }
    }

    if has_float {
        Ok(Value::Float(max_float))
    } else {
        Ok(Value::Integer(max_int))
    }
}

/// inc - インクリメント
/// 整数・浮動小数の両方に対応
pub fn native_inc(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["inc"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n + 1)),
        Value::Float(f) => Ok(Value::Float(f + 1.0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["inc", "numbers"])),
    }
}

/// dec - デクリメント
/// 整数・浮動小数の両方に対応
pub fn native_dec(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["dec"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n - 1)),
        Value::Float(f) => Ok(Value::Float(f - 1.0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["dec", "numbers"])),
    }
}

/// sum - 合計
/// 整数・浮動小数の両方に対応。浮動小数が1つでも含まれる場合は浮動小数で返す
pub fn native_sum(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sum"]));
    }
    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let mut int_sum = 0i64;
            let mut float_sum = 0.0f64;
            let mut has_float = false;

            for item in items {
                match item {
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
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["sum (elements)", "numbers"])),
                }
            }

            if has_float {
                Ok(Value::Float(float_sum))
            } else {
                Ok(Value::Integer(int_sum))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sum", "lists or vectors"])),
    }
}

// ========================================
// 比較演算（6個）
// ========================================

/// 値の等価性を判定するヘルパー関数
/// ListとVectorは内容が同じなら等しいと見なす
/// IntegerとFloatは数値的に等価なら等しいと見なす（Lisp系言語の一般的な仕様）
fn values_equal(a: &Value, b: &Value) -> bool {
    use std::ptr;
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        // IntegerとFloatの相互比較（数値的等価性）
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            (*a as f64 - b).abs() < f64::EPSILON
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bytes(a), Value::Bytes(b)) => a.as_ref() == b.as_ref(),
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
/// @qi-doc:category core/numeric
/// @qi-doc:functions +, -, *, /, %, abs, min, max, inc, dec, sum, =, <, >, <=, >=
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
