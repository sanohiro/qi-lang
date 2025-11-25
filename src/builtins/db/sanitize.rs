use super::*;
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::with_global;

/// 文字列値をサニタイズしてSQLインジェクションを防ぐ
///
/// # 引数
/// - `conn_id` (DbConnection): データベース接続
/// - `value` (string): サニタイズする文字列
///
/// # 戻り値
/// - (string): サニタイズされた文字列
///
/// # 注
/// バインドパラメータを使う方が推奨されます。この関数は必要に応じてのみ使用してください。
pub fn native_sanitize(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "db/sanitize");

    let conn_id = extract_conn_id(&args[0])?;
    let value = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/sanitize", "string"])),
    };

    let conn = with_global!(CONNECTIONS, &conn_id, MsgKey::DbConnectionNotFound);

    Ok(Value::String(conn.sanitize(value)))
}

/// db/sanitize-identifier - 識別子をサニタイズ
pub fn native_sanitize_identifier(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "db/sanitize-identifier");

    let conn_id = extract_conn_id(&args[0])?;
    let name = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/sanitize-identifier", "string"],
            ))
        }
    };

    let conn = with_global!(CONNECTIONS, &conn_id, MsgKey::DbConnectionNotFound);

    Ok(Value::String(conn.sanitize_identifier(name)))
}

/// db/escape-like - LIKE句のパターンをエスケープ
pub fn native_escape_like(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "db/escape-like");

    let conn_id = extract_conn_id(&args[0])?;
    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/escape-like", "string"],
            ))
        }
    };

    let conn = with_global!(CONNECTIONS, &conn_id, MsgKey::DbConnectionNotFound);

    Ok(Value::String(conn.escape_like(pattern)))
}
