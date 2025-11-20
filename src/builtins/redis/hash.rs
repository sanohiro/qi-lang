//! Redis hash operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

pub fn native_redis_hset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-hset", "4"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hset (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hset (key)", "strings"],
            ))
        }
    };

    let field = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hset (field)", "strings"],
            ))
        }
    };

    let value = match &args[3] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &[
                    "kvs/redis-hset (value)",
                    "strings, integers, floats, or bools",
                ],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| {
            let value = value.clone();
            async move { conn.hset(key, field, &value).await }
        })
        .await;

        match result {
            Ok(created) => Ok(Value::Bool(created)),
            Err(e) => Ok(Value::error(format!("Hset error: {}", e))),
        }
    })
}

/// kvs/redis-hget - ハッシュのフィールドから値を取得
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - field: フィールド名
///
/// 戻り値: 値 or nil or {:error message}
pub fn native_redis_hget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-hget", "3"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hget (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hget (key)", "strings"],
            ))
        }
    };

    let field = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hget (field)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.hget(key, field).await }).await;

        match result {
            Ok(Some(value)) => Ok(Value::String(value)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Ok(Value::error(format!("Hget error: {}", e))),
        }
    })
}

/// kvs/redis-hgetall - ハッシュ全体を取得
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: マップ {:field1 "value1" :field2 "value2"} or {:error message}
pub fn native_redis_hgetall(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-hgetall", "2"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hgetall (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-hgetall (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<Vec<(String, String)>> =
            execute_with_retry(url, |mut conn| async move { conn.hgetall(key).await }).await;

        match result {
            Ok(pairs) => {
                let mut map = crate::new_hashmap();
                for (field, value) in pairs {
                    // 文字列キーでマップに追加
                    map.insert(crate::value::MapKey::String(field), Value::String(value));
                }
                Ok(Value::Map(map))
            }
            Err(e) => Ok(Value::error(format!("Hgetall error: {}", e))),
        }
    })
}
