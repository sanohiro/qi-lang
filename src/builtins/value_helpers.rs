//! Value型の抽出・変換ヘルパー関数
//!
//! ビルトイン関数でのValue型チェックとデータ抽出を統一するヘルパー関数群。
//! パターンマッチの重複を削減し、エラーメッセージを統一します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
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
) -> Result<HashMap<String, Value>, String> {
    args.get(idx)
        .and_then(|v| match v {
            Value::Map(map) => Some(map.clone()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::ArgMustBeType, &[func, "map"]))
}

/// 引数からKeyword型を抽出
pub fn get_keyword_arg(args: &[Value], idx: usize, func: &str) -> Result<String, String> {
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
