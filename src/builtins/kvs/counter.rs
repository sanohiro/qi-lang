use super::*;

/// kvs/incr - キーの値をインクリメント
pub fn native_incr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/incr", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;

    // ドライバーをクローンしてからミューテックスを解放（ネットワークI/O前）
    let driver = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    match driver.incr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/decr - キーの値をデクリメント
pub fn native_decr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/decr", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;

    // ドライバーをクローンしてからミューテックスを解放（ネットワークI/O前）
    let driver = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    match driver.decr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}
