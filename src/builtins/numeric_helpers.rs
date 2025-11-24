//! 算術演算の共通ヘルパー関数
//!
//! オーバーフロー/アンダーフローチェック付きの整数演算を提供します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// 整数加算のオーバーフローチェック付きヘルパー
///
/// # 引数
/// - `a`: 被加算数
/// - `b`: 加算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 加算結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_int_add(a: i64, b: i64, op_name: &str) -> Result<i64, String> {
    a.checked_add(b)
        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &[op_name]))
}

/// 整数減算のアンダーフローチェック付きヘルパー
///
/// # 引数
/// - `a`: 被減算数
/// - `b`: 減算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 減算結果
/// - `Err(String)`: アンダーフロー時のエラーメッセージ
#[inline]
pub fn checked_int_sub(a: i64, b: i64, op_name: &str) -> Result<i64, String> {
    a.checked_sub(b)
        .ok_or_else(|| fmt_msg(MsgKey::IntegerUnderflow, &[op_name]))
}

/// 整数乗算のオーバーフローチェック付きヘルパー
///
/// # 引数
/// - `a`: 被乗算数
/// - `b`: 乗算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 乗算結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_int_mul(a: i64, b: i64, op_name: &str) -> Result<i64, String> {
    a.checked_mul(b)
        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &[op_name]))
}

/// 整数絶対値のオーバーフローチェック付きヘルパー
///
/// # 引数
/// - `n`: 整数値
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 絶対値
/// - `Err(String)`: オーバーフロー時のエラーメッセージ（i64::MINの場合）
#[inline]
pub fn checked_int_abs(n: i64, op_name: &str) -> Result<i64, String> {
    n.checked_abs()
        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &[op_name]))
}

/// 整数符号反転のオーバーフローチェック付きヘルパー
///
/// # 引数
/// - `n`: 整数値
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(i64)`: 符号反転結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ（i64::MINの場合）
#[inline]
pub fn checked_int_neg(n: i64, op_name: &str) -> Result<i64, String> {
    n.checked_neg()
        .ok_or_else(|| fmt_msg(MsgKey::IntegerOverflow, &[op_name]))
}

/// 整数加算（Value型を返す）
///
/// # 引数
/// - `a`: 被加算数
/// - `b`: 加算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 加算結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_add_value(a: i64, b: i64, op_name: &str) -> Result<Value, String> {
    checked_int_add(a, b, op_name).map(Value::Integer)
}

/// 整数減算（Value型を返す）
///
/// # 引数
/// - `a`: 被減算数
/// - `b`: 減算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 減算結果
/// - `Err(String)`: アンダーフロー時のエラーメッセージ
#[inline]
pub fn checked_sub_value(a: i64, b: i64, op_name: &str) -> Result<Value, String> {
    checked_int_sub(a, b, op_name).map(Value::Integer)
}

/// 整数乗算（Value型を返す）
///
/// # 引数
/// - `a`: 被乗算数
/// - `b`: 乗算数
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 乗算結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_mul_value(a: i64, b: i64, op_name: &str) -> Result<Value, String> {
    checked_int_mul(a, b, op_name).map(Value::Integer)
}

/// 整数絶対値（Value型を返す）
///
/// # 引数
/// - `n`: 整数値
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 絶対値
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_abs_value(n: i64, op_name: &str) -> Result<Value, String> {
    checked_int_abs(n, op_name).map(Value::Integer)
}

/// 整数符号反転（Value型を返す）
///
/// # 引数
/// - `n`: 整数値
/// - `op_name`: 演算子名（エラーメッセージ用）
///
/// # 戻り値
/// - `Ok(Value::Integer)`: 符号反転結果
/// - `Err(String)`: オーバーフロー時のエラーメッセージ
#[inline]
pub fn checked_neg_value(n: i64, op_name: &str) -> Result<Value, String> {
    checked_int_neg(n, op_name).map(Value::Integer)
}
