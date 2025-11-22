use super::*;
use crate::i18n::{fmt_msg, MsgKey};

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/sanitize"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let value = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/sanitize", "string"])),
    };

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    Ok(Value::String(conn.sanitize(value)))
}

/// db/sanitize-identifier - 識別子をサニタイズ
pub fn native_sanitize_identifier(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/sanitize-identifier"]));
    }

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

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    Ok(Value::String(conn.sanitize_identifier(name)))
}

/// db/escape-like - LIKE句のパターンをエスケープ
pub fn native_escape_like(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/escape-like"]));
    }

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

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    Ok(Value::String(conn.escape_like(pattern)))
}
