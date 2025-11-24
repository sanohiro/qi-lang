//! Value型の抽出・変換ヘルパー関数
//!
//! ビルトイン関数でのValue型チェックとデータ抽出を統一するヘルパー関数群。
//! パターンマッチの重複を削減し、エラーメッセージを統一します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{MapKey, Value};
use crate::HashMap;

/// 引数からString型を抽出
///
/// # 使用例
///
/// ```
/// use qi_lang::value::Value;
/// use qi_lang::builtins::value_helpers::get_string_arg;
///
/// let args = vec![Value::String("hello".to_string())];
/// let s = get_string_arg(&args, 0, "split").unwrap();
/// assert_eq!(s, "hello");
/// ```
pub fn get_string_arg(args: &[Value], idx: usize, func: &str) -> Result<String, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "string"]))
}

/// 引数からString型の参照を抽出（クローンなし）
pub fn get_string_ref<'a>(args: &'a [Value], idx: usize, func: &str) -> Result<&'a str, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "string"]))
}

/// 引数からInteger型を抽出
pub fn get_int_arg(args: &[Value], idx: usize, func: &str) -> Result<i64, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Integer(n) => Some(*n),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"]))
}

/// 引数からFloat型を抽出
pub fn get_float_arg(args: &[Value], idx: usize, func: &str) -> Result<f64, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "number"]))
}

/// 引数からBool型を抽出
pub fn get_bool_arg(args: &[Value], idx: usize, func: &str) -> Result<bool, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "boolean"]))
}

/// 引数からVector型を抽出
pub fn get_vector_arg(args: &[Value], idx: usize, func: &str) -> Result<im::Vector<Value>, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Vector(vec) => Some(vec.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "vector"]))
}

/// 引数からList型を抽出
pub fn get_list_arg(args: &[Value], idx: usize, func: &str) -> Result<im::Vector<Value>, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::List(list) => Some(list.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "list"]))
}

/// 引数からMap型を抽出
pub fn get_map_arg(
    args: &[Value],
    idx: usize,
    func: &str,
) -> Result<HashMap<MapKey, Value>, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Map(map) => Some(map.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "map"]))
}

/// 引数からKeyword型を抽出
pub fn get_keyword_arg(
    args: &[Value],
    idx: usize,
    func: &str,
) -> Result<std::sync::Arc<str>, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Keyword(kw) => Some(kw.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "keyword"]))
}

/// 引数から数値（IntegerまたはFloat）を抽出
pub fn get_number_arg(args: &[Value], idx: usize, func: &str) -> Result<f64, String> {
    get_float_arg(args, idx, func)
}

/// 引数からコレクション（Vector、List、またはMap）のサイズを取得
pub fn get_collection_size(value: &Value) -> Option<usize> {
    match value {
        Value::Vector(v) => Some(v.len()),
        Value::List(l) => Some(l.len()),
        Value::Map(m) => Some(m.len()),
        Value::String(s) => Some(s.len()),
        _ => None,
    }
}

/// Valueが真値かどうかを判定（nil と false 以外は真）
pub fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Nil | Value::Bool(false))
}

/// 2つのValueが等価かどうかを判定
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            (*a as f64 - b).abs() < f64::EPSILON
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Keyword(a), Value::Keyword(b)) => a == b,
        (Value::Vector(a), Value::Vector(b)) => a == b,
        (Value::List(a), Value::List(b)) => a == b,
        (Value::Map(a), Value::Map(b)) => a == b,
        _ => false,
    }
}

// ========================================
// 範囲検証・型変換ヘルパー関数
// ========================================

/// ポート番号検証（0-65535）
///
/// # 引数
/// - `val`: 検証対象の値
/// - `func`: 関数名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(u16)`: 有効なポート番号
/// - `Err(String)`: 範囲外またはinteger型でない場合のエラーメッセージ
#[inline]
pub fn validate_port(val: &Value, func: &str) -> Result<u16, String> {
    match val {
        Value::Integer(p) if *p >= 0 && *p <= u16::MAX as i64 => Ok(*p as u16),
        Value::Integer(p) => Err(fmt_msg(MsgKey::ServerInvalidPortNumber, &[&p.to_string()])),
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"])),
    }
}

/// バイト値検証（0-255）
///
/// # 引数
/// - `val`: 検証対象の値
/// - `func`: 関数名（エラーメッセージ用）
/// - `idx`: インデックス（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(u8)`: 有効なバイト値
/// - `Err(String)`: 範囲外またはinteger型でない場合のエラーメッセージ
#[inline]
pub fn validate_byte(val: &Value, func: &str, idx: usize) -> Result<u8, String> {
    match val {
        Value::Integer(n) if *n >= 0 && *n <= u8::MAX as i64 => Ok(*n as u8),
        Value::Integer(n) => Err(fmt_msg(
            MsgKey::ByteOutOfRange,
            &[&idx.to_string(), &n.to_string()],
        )),
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"])),
    }
}

/// 正の整数をusizeに変換（>0）
///
/// # 引数
/// - `val`: 検証対象の値
/// - `func`: 関数名（エラーメッセージ用）
/// - `param`: パラメータ名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(usize)`: 変換後の正の整数
/// - `Err(String)`: 負数、ゼロ、またはオーバーフローの場合のエラーメッセージ
#[inline]
pub fn to_positive_usize(val: &Value, func: &str, param: &str) -> Result<usize, String> {
    match val {
        Value::Integer(n) if *n > 0 => usize::try_from(*n)
            .map_err(|_| fmt_msg(MsgKey::IntegerOverflow, &[&format!("{} {}", func, param)])),
        Value::Integer(n) => Err(fmt_msg(
            MsgKey::MustBePositive,
            &[func, param, &n.to_string()],
        )),
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"])),
    }
}

/// 非負整数をusizeに変換（>=0）
///
/// # 引数
/// - `val`: 検証対象の値
/// - `func`: 関数名（エラーメッセージ用）
/// - `param`: パラメータ名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(usize)`: 変換後の非負整数
/// - `Err(String)`: 負数またはオーバーフローの場合のエラーメッセージ
#[inline]
pub fn to_nonnegative_usize(val: &Value, func: &str, param: &str) -> Result<usize, String> {
    match val {
        Value::Integer(n) if *n >= 0 => usize::try_from(*n)
            .map_err(|_| fmt_msg(MsgKey::IntegerOverflow, &[&format!("{} {}", func, param)])),
        Value::Integer(n) => Err(fmt_msg(
            MsgKey::MustBeNonNegative,
            &[func, param, &n.to_string()],
        )),
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"])),
    }
}

/// 範囲指定の整数検証（min <= n <= max）
///
/// # 引数
/// - `val`: 検証対象の値
/// - `min`: 最小値（含む）
/// - `max`: 最大値（含む）
/// - `func`: 関数名（エラーメッセージ用）
/// - `param`: パラメータ名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 範囲内の整数値
/// - `Err(String)`: 範囲外またはinteger型でない場合のエラーメッセージ
#[inline]
pub fn validate_int_range(
    val: &Value,
    min: i64,
    max: i64,
    func: &str,
    param: &str,
) -> Result<i64, String> {
    match val {
        Value::Integer(n) if *n >= min && *n <= max => Ok(*n),
        Value::Integer(n) => Err(fmt_msg(
            MsgKey::IntegerOutOfRange,
            &[
                func,
                param,
                &n.to_string(),
                &min.to_string(),
                &max.to_string(),
            ],
        )),
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &[func, "integer"])),
    }
}

/// usizeをi64に安全変換（Value::Integerを返す）
///
/// # 引数
/// - `n`: 変換対象のusize値
/// - `func`: 関数名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 変換後の整数値
/// - `Err(String)`: オーバーフローの場合のエラーメッセージ
#[inline]
pub fn usize_to_int_value(n: usize, func: &str) -> Result<Value, String> {
    i64::try_from(n)
        .map(Value::Integer)
        .map_err(|_| fmt_msg(MsgKey::IntegerOverflow, &[func]))
}

/// i64をusizeに安全変換
///
/// # 引数
/// - `n`: 変換対象のi64値
/// - `func`: 関数名（エラーメッセージ用）
/// - `param`: パラメータ名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(usize)`: 変換後のusize値
/// - `Err(String)`: オーバーフローの場合のエラーメッセージ
#[inline]
pub fn i64_to_usize(n: i64, func: &str, param: &str) -> Result<usize, String> {
    usize::try_from(n)
        .map_err(|_| fmt_msg(MsgKey::IntegerOverflow, &[&format!("{} {}", func, param)]))
}

/// i64をu16に安全変換
///
/// # 引数
/// - `n`: 変換対象のi64値
/// - `func`: 関数名（エラーメッセージ用）
/// - `param`: パラメータ名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(u16)`: 変換後のu16値
/// - `Err(String)`: オーバーフローの場合のエラーメッセージ
#[inline]
pub fn i64_to_u16(n: i64, func: &str, param: &str) -> Result<u16, String> {
    u16::try_from(n)
        .map_err(|_| fmt_msg(MsgKey::IntegerOverflow, &[&format!("{} {}", func, param)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_string_arg() {
        let args = vec![Value::String("hello".to_string()), Value::Integer(42)];

        let result = get_string_arg(&args, 0, "test");
        assert_eq!(result.unwrap(), "hello");

        let result = get_string_arg(&args, 1, "test");
        assert!(result.is_err());

        let result = get_string_arg(&args, 2, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_int_arg() {
        let args = vec![Value::Integer(42), Value::String("hello".to_string())];

        let result = get_int_arg(&args, 0, "test");
        assert_eq!(result.unwrap(), 42);

        let result = get_int_arg(&args, 1, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_float_arg() {
        let args = vec![
            Value::Float(std::f64::consts::PI),
            Value::Integer(42),
            Value::String("hello".to_string()),
        ];

        assert_eq!(
            get_float_arg(&args, 0, "test").unwrap(),
            std::f64::consts::PI
        );
        assert_eq!(get_float_arg(&args, 1, "test").unwrap(), 42.0);
        assert!(get_float_arg(&args, 2, "test").is_err());
    }

    #[test]
    fn test_is_truthy() {
        assert!(!is_truthy(&Value::Nil));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_truthy(&Value::Integer(0)));
        assert!(is_truthy(&Value::String("".to_string())));
    }

    #[test]
    fn test_values_equal() {
        assert!(values_equal(&Value::Nil, &Value::Nil));
        assert!(values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(values_equal(&Value::Integer(42), &Value::Float(42.0)));
        assert!(!values_equal(&Value::Integer(42), &Value::Integer(43)));
    }
}
