use super::*;
use crate::check_args;
use crate::with_global;

/// kvs/incr - キーの値をインクリメント
pub fn native_incr(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/incr");

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let driver = with_global!(CONNECTIONS, &conn_id, MsgKey::ConnectionNotFound);

    match driver.incr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/decr - キーの値をデクリメント
pub fn native_decr(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/decr");

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let driver = with_global!(CONNECTIONS, &conn_id, MsgKey::ConnectionNotFound);

    match driver.decr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}
