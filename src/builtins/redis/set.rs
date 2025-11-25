//! Redis set operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

/// kvs/redis-sadd - セットにメンバーを追加
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: キー名
/// - member: 追加するメンバー
///
/// 戻り値: 追加されたメンバー数 or {:error message}
pub fn native_redis_sadd(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "kvs/redis-sadd");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-sadd (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-sadd (key)", "strings"],
            ))
        }
    };

    let member = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &[
                    "kvs/redis-sadd (member)",
                    "strings, integers, floats, or bools",
                ],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| {
            let member = member.clone();
            async move { conn.sadd(key, &member).await }
        })
        .await;

        match result {
            Ok(count) => Ok(Value::Integer(count)),
            Err(e) => Ok(Value::error(format!("Sadd error: {}", e))),
        }
    })
}

/// kvs/redis-smembers - セットの全メンバーを取得
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: メンバーのベクタ or {:error message}
pub fn native_redis_smembers(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-smembers");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-smembers (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-smembers (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<Vec<String>> =
            execute_with_retry(url, |mut conn| async move { conn.smembers(key).await }).await;

        match result {
            Ok(members) => Ok(Value::Vector(
                members
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(e) => Ok(Value::error(format!("Smembers error: {}", e))),
        }
    })
}
