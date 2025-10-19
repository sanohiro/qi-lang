//! Core述語・型判定関数
//!
//! 型チェック（9個）: nil?, list?, vector?, map?, string?, integer?, float?, number?, keyword?, function?, atom?
//! コレクション（3個）: coll?, sequential?, empty?
//! 状態（3個）: some?, true?, false?
//! 数値（5個）: even?, odd?, positive?, negative?, zero?
//! 合計20個のCore関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

// ========================================
// 型チェック（11個）
// ========================================

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

/// number? - 数値判定
pub fn native_number_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["number?"]));
    }
    Ok(Value::Bool(matches!(
        args[0],
        Value::Integer(_) | Value::Float(_)
    )))
}

/// keyword? - キーワードかどうか判定
pub fn native_keyword_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["keyword?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Keyword(_))))
}

/// function? - 関数判定
pub fn native_function_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["function?"]));
    }
    Ok(Value::Bool(matches!(
        args[0],
        Value::Function(_) | Value::NativeFunc(_)
    )))
}

/// atom? - atom判定
pub fn native_atom_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["atom?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Atom(_))))
}

// ========================================
// コレクション（3個）
// ========================================

/// coll? - コレクション型かどうか判定
pub fn native_coll_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["coll?"]));
    }
    Ok(Value::Bool(matches!(
        args[0],
        Value::List(_) | Value::Vector(_) | Value::Map(_)
    )))
}

/// sequential? - シーケンシャル型かどうか判定
pub fn native_sequential_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sequential?"]));
    }
    Ok(Value::Bool(matches!(
        args[0],
        Value::List(_) | Value::Vector(_)
    )))
}

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
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["empty?", "strings or collections"],
        )),
    }
}

// ========================================
// 状態（3個）
// ========================================

/// some? - nilでないかどうか判定
pub fn native_some_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["some?"]));
    }
    Ok(Value::Bool(!matches!(args[0], Value::Nil)))
}

/// true? - 厳密にtrueかどうか判定
pub fn native_true_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["true?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Bool(true))))
}

/// false? - 厳密にfalseかどうか判定
pub fn native_false_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["false?"]));
    }
    Ok(Value::Bool(matches!(args[0], Value::Bool(false))))
}

// ========================================
// 数値（5個）
// ========================================

/// even? - 偶数かどうか判定
pub fn native_even_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["even?"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Bool(n % 2 == 0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["even?", "integers"])),
    }
}

/// odd? - 奇数かどうか判定
pub fn native_odd_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["odd?"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Bool(n % 2 != 0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["odd?", "integers"])),
    }
}

/// positive? - 正の数かどうか判定
pub fn native_positive_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["positive?"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Bool(*n > 0)),
        Value::Float(f) => Ok(Value::Bool(*f > 0.0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["positive?", "numbers"])),
    }
}

/// negative? - 負の数かどうか判定
pub fn native_negative_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["negative?"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Bool(*n < 0)),
        Value::Float(f) => Ok(Value::Bool(*f < 0.0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["negative?", "numbers"])),
    }
}

/// zero? - ゼロかどうか判定
pub fn native_zero_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["zero?"]));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Bool(*n == 0)),
        Value::Float(f) => Ok(Value::Bool(*f == 0.0)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["zero?", "numbers"])),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category core/predicates
/// @qi-doc:functions nil?, list?, vector?, map?, string?, integer?, float?, number?, keyword?, function?, atom?, coll?, sequential?, empty?, some?, true?, false?, even?, odd?, positive?, negative?, zero?
pub const FUNCTIONS: super::NativeFunctions = &[
    // 型チェック
    ("nil?", native_nil),
    ("list?", native_list_q),
    ("vector?", native_vector_q),
    ("map?", native_map_q),
    ("string?", native_string_q),
    ("integer?", native_integer_q),
    ("float?", native_float_q),
    ("number?", native_number_q),
    ("keyword?", native_keyword_q),
    ("function?", native_function_q),
    ("atom?", native_atom_q),
    // コレクション
    ("coll?", native_coll_q),
    ("sequential?", native_sequential_q),
    ("empty?", native_empty),
    // 状態
    ("some?", native_some_q),
    ("true?", native_true_q),
    ("false?", native_false_q),
    // 数値
    ("even?", native_even_q),
    ("odd?", native_odd_q),
    ("positive?", native_positive_q),
    ("negative?", native_negative_q),
    ("zero?", native_zero_q),
];
