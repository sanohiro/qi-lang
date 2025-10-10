//! エラー処理関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// error - 回復不能なエラーを投げる
///
/// 引数:
/// - message: エラーメッセージ（文字列）
///
/// 戻り値:
/// - 常にエラーを返す
///
/// 例:
/// ```lisp
/// (error "something went wrong")
/// (error "file not found")
/// ```
pub fn native_error(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["error"]));
    }

    match &args[0] {
        Value::String(msg) => Err(msg.clone()),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["error", "strings"])),
    }
}
