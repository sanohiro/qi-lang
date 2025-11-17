use super::*;

/// kvs/lpush - リスト左端に要素を追加
pub fn native_lpush(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lpush", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpush (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpush (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/lpush (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.lpush(key, &value) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/rpush - リスト右端に要素を追加
pub fn native_rpush(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/rpush", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpush (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpush (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/rpush (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.rpush(key, &value) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/lpop - リスト左端から要素を取得
pub fn native_lpop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lpop", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpop (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpop (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.lpop(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/rpop - リスト右端から要素を取得
pub fn native_rpop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/rpop", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpop (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpop (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.rpop(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/lrange - リストの範囲を取得
pub fn native_lrange(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lrange", "4"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (key)", "strings"])),
    };

    let start = match &args[2] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/lrange (start)", "integers"],
            ))
        }
    };

    let stop = match &args[3] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/lrange (stop)", "integers"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    match driver.lrange(key, start, stop) {
        Ok(items) => Ok(Value::Vector(
            items
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

// ========================================
// 関数登録テーブル
