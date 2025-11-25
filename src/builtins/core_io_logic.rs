//! Core I/O・論理・エラー関数
//!
//! I/O（2個）: print, println
//! 論理（1個）: not
//! エラー（1個）: error
//! 合計4個のCore関数

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

// ========================================
// I/O（2個）
// ========================================

/// print - 改行なしで出力
pub fn native_print(args: &[Value]) -> Result<Value, String> {
    let output = if args.is_empty() {
        String::new()
    } else {
        args.iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => format!("{:?}", v),
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    print!("{}", output);
    Ok(Value::Nil)
}

/// println - 改行付きで出力
pub fn native_println(args: &[Value]) -> Result<Value, String> {
    let output = if args.is_empty() {
        String::new()
    } else {
        args.iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => format!("{:?}", v),
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!("{}", output);
    Ok(Value::Nil)
}

// ========================================
// 論理（1個）
// ========================================

/// not - 論理否定
pub fn native_not(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "not");
    Ok(Value::Bool(!args[0].is_truthy()))
}

// ========================================
// エラー（1個）
// ========================================

/// error - 回復不能なエラーを投げる
///
/// 引数:
/// - message: エラーメッセージ（文字列）
///
/// 戻り値:
/// - 常にエラーを返す
///
/// 例:
/// ```qi
/// (error "something went wrong")
/// (error "file not found")
/// ```
pub fn native_error(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "error");

    match &args[0] {
        Value::String(msg) => Err(msg.clone()),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["error", "strings"])),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category core/io-logic
/// @qi-doc:functions print, println, not, error
pub const FUNCTIONS: super::NativeFunctions = &[
    ("print", native_print),
    ("println", native_println),
    ("not", native_not),
    ("error", native_error),
];
