use super::*;

#[cfg(feature = "kvs-redis")]
use super::redis_driver::RedisDriver;

/// kvs/connect - KVSに接続
///
/// URLからバックエンドを自動判別し、接続IDを返す。
///
/// # 引数
/// - url: 接続URL（例: "redis://localhost:6379"）
///
/// # 戻り値
/// - 接続ID（文字列）
pub fn native_connect(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "kvs/connect");

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/connect (url)", "strings"])),
    };

    // URLからバックエンドを判定
    let driver: Box<dyn KvsDriver> = if url.starts_with("redis://") {
        #[cfg(feature = "kvs-redis")]
        {
            Box::new(RedisDriver::new(url))
        }
        #[cfg(not(feature = "kvs-redis"))]
        {
            return Err(fmt_msg(MsgKey::RedisSupportNotEnabled, &[]));
        }
    } else {
        return Err(fmt_msg(MsgKey::UnsupportedKvsUrl, &[url]));
    };

    // 接続を保存（Arc で包んでロック解放を可能にする）
    let conn_id = gen_conn_id();
    CONNECTIONS
        .lock()
        .insert(conn_id.clone(), std::sync::Arc::new(driver));

    Ok(Value::String(format!("KvsConnection:{}", conn_id)))
}

/// kvs/close - 接続をクローズ
///
/// # 引数
/// - conn: 接続ID
///
/// # 戻り値
/// - nil
pub fn native_close(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "kvs/close");

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/close (conn)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;

    // 接続を削除
    let mut connections = CONNECTIONS.lock();
    connections
        .remove(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::ConnectionNotFound, &[&conn_id]))?;

    Ok(Value::Nil)
}

/// 接続IDから接続を取得
pub(super) fn get_connection(conn_str: &str) -> Result<String, String> {
    if !conn_str.starts_with("KvsConnection:") {
        return Err(fmt_msg(MsgKey::InvalidConnection, &[]));
    }
    let conn_id = &conn_str["KvsConnection:".len()..];

    // ロックを1回だけ取得して存在チェック（TOCTOU問題を回避）
    if CONNECTIONS.lock().contains_key(conn_id) {
        Ok(conn_id.to_string())
    } else {
        Err(fmt_msg(MsgKey::ConnectionNotFound, &[conn_id]))
    }
}
