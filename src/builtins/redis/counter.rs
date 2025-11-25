//! Redis counter operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

/// kvs/redis-incr - キーの値をインクリメント
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: キー名
///
/// 戻り値: インクリメント後の値 or {:error message}
pub fn native_redis_incr(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-incr");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-incr (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-incr (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.incr(key, 1).await }).await;

        match result {
            Ok(value) => Ok(Value::Integer(value)),
            Err(e) => Ok(Value::error(format!("Incr error: {}", e))),
        }
    })
}

/// kvs/redis-decr - キーの値をデクリメント
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: デクリメント後の値 or {:error message}
pub fn native_redis_decr(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-decr");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-decr (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-decr (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.decr(key, 1).await }).await;

        match result {
            Ok(value) => Ok(Value::Integer(value)),
            Err(e) => Ok(Value::error(format!("Decr error: {}", e))),
        }
    })
}
