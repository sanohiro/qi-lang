//! バイナリデータ操作関数
//!
//! バイナリデータの生成、変換、操作を提供します。

use crate::builtins::value_helpers::validate_byte;
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::sync::Arc;

/// bytes - Vector<Integer>からBytes型を生成
///
/// 引数:
/// - vector: 整数のベクタ（各要素は0-255の範囲）
///
/// 戻り値: Bytes型
///
/// 例:
/// ```qi
/// (bytes [255 254 253])  ;=> #bytes[FF FE FD]
/// (bytes [0 1 2 3])      ;=> #bytes[00 01 02 03]
/// ```
pub fn native_bytes(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "bytes");

    match &args[0] {
        Value::Vector(v) => {
            let mut bytes = Vec::with_capacity(v.len());
            for (i, item) in v.iter().enumerate() {
                let byte = validate_byte(item, "bytes", i)?;
                bytes.push(byte);
            }
            Ok(Value::Bytes(Arc::from(bytes)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["bytes", "vectors"])),
    }
}

/// bytes/to-vec - Bytes型をVector<Integer>に変換
///
/// 引数:
/// - bytes: Bytes型
///
/// 戻り値: Vector<Integer>（各要素は0-255）
///
/// 例:
/// ```qi
/// (bytes/to-vec #bytes[FF FE FD])  ;=> [255 254 253]
/// ```
pub fn native_bytes_to_vec(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "bytes/to-vec");

    match &args[0] {
        Value::Bytes(b) => {
            let vec: im::Vector<Value> =
                b.iter().map(|&byte| Value::Integer(byte as i64)).collect();
            Ok(Value::Vector(vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["bytes/to-vec", "bytes"])),
    }
}

/// 登録すべき関数のリスト
/// @qi-doc:category bytes
/// @qi-doc:functions bytes, bytes/to-vec
pub const FUNCTIONS: super::NativeFunctions = &[
    ("bytes", native_bytes),
    ("bytes/to-vec", native_bytes_to_vec),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        let vec = im::vector![Value::Integer(255), Value::Integer(254), Value::Integer(0),];
        let result = native_bytes(&[Value::Vector(vec)]).unwrap();

        match result {
            Value::Bytes(b) => {
                assert_eq!(b.as_ref(), &[255, 254, 0]);
            }
            _ => panic!("Expected Bytes, got {:?}", result),
        }
    }

    #[test]
    fn test_bytes_out_of_range() {
        let vec = im::vector![Value::Integer(256)];
        let result = native_bytes(&[Value::Vector(vec)]);
        assert!(result.is_err());

        let vec = im::vector![Value::Integer(-1)];
        let result = native_bytes(&[Value::Vector(vec)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_to_vec() {
        let bytes = Arc::from(vec![255u8, 254, 0]);
        let result = native_bytes_to_vec(&[Value::Bytes(bytes)]).unwrap();

        match result {
            Value::Vector(v) => {
                assert_eq!(v.len(), 3);
                assert_eq!(v[0], Value::Integer(255));
                assert_eq!(v[1], Value::Integer(254));
                assert_eq!(v[2], Value::Integer(0));
            }
            _ => panic!("Expected Vector, got {:?}", result),
        }
    }
}
