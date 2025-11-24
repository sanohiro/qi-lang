use super::*;
use crate::with_global;

/// kvs/mget - 複数のキーの値を一括取得
pub fn native_mget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/mget", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mget (conn)", "strings"])),
    };

    let keys = match &args[1] {
        Value::Vector(v) => v
            .iter()
            .map(|k| match k {
                Value::String(s) => Ok(s.clone()),
                _ => Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["kvs/mget (keys)", "vector of strings"],
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mget (keys)", "vectors"])),
    };

    let conn_id = get_connection(conn_str)?;
    let driver = with_global!(CONNECTIONS, &conn_id, MsgKey::ConnectionNotFound);

    match driver.mget(&keys) {
        Ok(values) => Ok(Value::Vector(
            values
                .into_iter()
                .map(|opt| match opt {
                    Some(s) => Value::String(s),
                    None => Value::Nil,
                })
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/mset - 複数のキーと値を一括設定
pub fn native_mset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/mset", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mset (conn)", "strings"])),
    };

    let pairs = match &args[1] {
        Value::Map(m) => {
            let mut map = HashMap::new();
            for (k, v) in m.iter() {
                // MapKeyから実際の文字列を抽出（to_string()はデバッグ形式になるため使わない）
                let key_str = match k {
                    crate::value::MapKey::Keyword(kw) => kw.to_string(),
                    crate::value::MapKey::String(s) => s.clone(),
                    crate::value::MapKey::Symbol(sym) => sym.to_string(),
                    crate::value::MapKey::Integer(i) => i.to_string(),
                };
                let value_str = match v {
                    Value::String(s) => s.clone(),
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["kvs/mset (values)", "strings, integers, floats, or bools"],
                        ))
                    }
                };
                map.insert(key_str, value_str);
            }
            map
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mset (pairs)", "maps"])),
    };

    let conn_id = get_connection(conn_str)?;
    let driver = with_global!(CONNECTIONS, &conn_id, MsgKey::ConnectionNotFound);

    match driver.mset(&pairs) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(e)),
    }
}
