use super::*;

/// kvs/sadd - セットにメンバーを追加
pub fn native_sadd(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/sadd", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/sadd (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/sadd (key)", "strings"])),
    };

    let member = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/sadd (member)", "strings, integers, floats, or bools"],
            ))
        }
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

    match driver.sadd(key, &member) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/smembers - セットの全メンバーを取得
pub fn native_smembers(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/smembers", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/smembers (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/smembers (key)", "strings"],
            ))
        }
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

    match driver.smembers(key) {
        Ok(members) => Ok(Value::Vector(
            members
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}
