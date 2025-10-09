//! 論理演算関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// not - 論理否定
pub fn native_not(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["not"]));
    }
    Ok(Value::Bool(!args[0].is_truthy()))
}
