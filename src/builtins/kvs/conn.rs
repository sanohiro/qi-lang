use super::*;

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
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/connect", "1"]));
    }

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

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), driver);

    Ok(Value::String(format!("KvsConnection:{}", conn_id)))
}

/// 接続IDから接続を取得
fn get_connection(conn_str: &str) -> Result<String, String> {
    if !conn_str.starts_with("KvsConnection:") {
        return Err(fmt_msg(MsgKey::InvalidConnection, &[]));
    }
    let conn_id = &conn_str["KvsConnection:".len()..];
    if !CONNECTIONS.lock().contains_key(conn_id) {
        return Err(fmt_msg(MsgKey::ConnectionNotFound, &[conn_id]));
    }
    Ok(conn_id.to_string())
}
