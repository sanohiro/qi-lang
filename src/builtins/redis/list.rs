//! Redis list operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

/// kvs/redis-lpush - リスト左端に要素を追加
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: キー名
/// - value: 追加する値
///
/// 戻り値: リスト長 or {:error message}
pub fn native_redis_lpush(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "kvs/redis-lpush");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lpush (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lpush (key)", "strings"],
            ))
        }
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &[
                    "kvs/redis-lpush (value)",
                    "strings, integers, floats, or bools",
                ],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| {
            let value = value.clone();
            async move { conn.lpush(key, &value).await }
        })
        .await;

        match result {
            Ok(len) => Ok(Value::Integer(len)),
            Err(e) => Ok(Value::error(format!("Lpush error: {}", e))),
        }
    })
}

/// kvs/redis-rpush - リスト右端に要素を追加
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - value: 追加する値
///
/// 戻り値: リスト長 or {:error message}
pub fn native_redis_rpush(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "kvs/redis-rpush");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-rpush (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-rpush (key)", "strings"],
            ))
        }
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &[
                    "kvs/redis-rpush (value)",
                    "strings, integers, floats, or bools",
                ],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| {
            let value = value.clone();
            async move { conn.rpush(key, &value).await }
        })
        .await;

        match result {
            Ok(len) => Ok(Value::Integer(len)),
            Err(e) => Ok(Value::error(format!("Rpush error: {}", e))),
        }
    })
}

/// kvs/redis-lpop - リスト左端から要素を取得
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: 値 or nil or {:error message}
pub fn native_redis_lpop(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-lpop");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lpop (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lpop (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.lpop(key, None).await }).await;

        match result {
            Ok(Some(value)) => Ok(Value::String(value)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Ok(Value::error(format!("Lpop error: {}", e))),
        }
    })
}

/// kvs/redis-rpop - リスト右端から要素を取得
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: 値 or nil or {:error message}
pub fn native_redis_rpop(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-rpop");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-rpop (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-rpop (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.rpop(key, None).await }).await;

        match result {
            Ok(Some(value)) => Ok(Value::String(value)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Ok(Value::error(format!("Rpop error: {}", e))),
        }
    })
}
