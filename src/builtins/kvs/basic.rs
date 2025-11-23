use super::*;

/// kvs/get - キーの値を取得
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/get", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/get (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/get (key)", "strings"])),
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

    match driver.get(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/set - キーに値を設定
pub fn native_set(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/set", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/set (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/set (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/set (value)", "strings, integers, floats, or bools"],
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

    match driver.set(key, &value) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/delete - キーを削除
pub fn native_delete(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/delete", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/delete (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/delete (key)", "strings"])),
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

    match driver.delete(key) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/exists? - キーが存在するかチェック
pub fn native_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/exists?", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/exists? (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/exists? (key)", "strings"])),
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

    match driver.exists(key) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/keys - パターンにマッチするキー一覧を取得
pub fn native_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/keys", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/keys (conn)", "strings"])),
    };

    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/keys (pattern)", "strings"],
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

    match driver.keys(pattern) {
        Ok(keys) => Ok(Value::Vector(
            keys.into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/expire - キーに有効期限を設定（秒）
pub fn native_expire(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/expire", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/expire (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/expire (key)", "strings"])),
    };

    let seconds = match &args[2] {
        Value::Integer(i) => {
            if *i < 0 {
                return Err(fmt_msg(
                    MsgKey::MustBeNonNegative,
                    &["kvs/expire", "seconds"],
                ));
            }
            *i
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/expire (seconds)", "integers"],
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

    match driver.expire(key, seconds) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/ttl - キーの残り有効期限を取得（秒）
pub fn native_ttl(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/ttl", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/ttl (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/ttl (key)", "strings"])),
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

    match driver.ttl(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}
