use super::*;

/// kvs/hset - ハッシュのフィールドに値を設定
pub fn native_hset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hset", "4"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (key)", "strings"])),
    };

    let field = match &args[2] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (field)", "strings"])),
    };

    let value = match &args[3] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/hset (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.hset(key, field, &value) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/hget - ハッシュのフィールドから値を取得
pub fn native_hget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hget", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (key)", "strings"])),
    };

    let field = match &args[2] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (field)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.hget(key, field) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/hgetall - ハッシュ全体を取得
pub fn native_hgetall(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hgetall", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/hgetall (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hgetall (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.hgetall(key) {
        Ok(pairs) => {
            let mut map = crate::new_hashmap();
            for (field, value) in pairs {
                // 文字列キーでマップに追加
                map.insert(field, Value::String(value));
            }
            Ok(Value::Map(map))
        }
        Err(e) => Ok(Value::error(e)),
    }
}

