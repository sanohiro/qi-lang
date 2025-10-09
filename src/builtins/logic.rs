//! 論理演算関数

use crate::value::Value;

/// not - 論理否定
pub fn native_not(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("notには1つの引数が必要です".to_string());
    }
    Ok(Value::Bool(!args[0].is_truthy()))
}
