//! Redis multi operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

pub fn native_redis_mget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-mget", "2"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-mget (url)", "strings"],
            ))
        }
    };

    let keys = match &args[1] {
        Value::Vector(vec) => vec
            .iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                _ => Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["kvs/redis-mget (keys)", "vector of strings"],
                )),
            })
            .collect::<Result<Vec<String>, String>>()?,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-mget (keys)", "vector"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<Vec<redis::Value>> = execute_with_retry(url, |mut conn| {
            let keys = keys.to_vec();
            async move {
                let mut cmd = redis::cmd("MGET");
                for key in &keys {
                    cmd.arg(key);
                }
                cmd.query_async(&mut conn).await
            }
        })
        .await;

        match result {
            Ok(values) => Ok(Value::Vector(
                values
                    .into_iter()
                    .map(|v| match v {
                        redis::Value::Nil => Value::Nil,
                        redis::Value::BulkString(bytes) => String::from_utf8(bytes)
                            .map(Value::String)
                            .unwrap_or(Value::Nil),
                        redis::Value::SimpleString(s) => Value::String(s),
                        _ => Value::Nil,
                    })
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(e) => Ok(Value::error(format!("Mget error: {}", e))),
        }
    })
}

/// kvs/redis-mset - 複数のキーと値を一括設定
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - pairs: キーと値のペアのマップ
///
/// 戻り値: "OK" or {:error message}
pub fn native_redis_mset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-mset", "2"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-mset (url)", "strings"],
            ))
        }
    };

    let pairs = match &args[1] {
        Value::Map(m) => m
            .iter()
            .map(|(k, v)| match v {
                Value::String(s) => {
                    // キーから引用符を取り除く（もし含まれている場合）
                    let clean_key = k.trim_matches('"');
                    Ok((clean_key.to_string(), s.clone()))
                }
                _ => Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["kvs/redis-mset (values)", "strings"],
                )),
            })
            .collect::<Result<Vec<(String, String)>, String>>()?,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-mset (pairs)", "map"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<String> = execute_with_retry(url, |mut conn| {
            let pairs = pairs.clone();
            async move {
                let mut cmd = redis::cmd("MSET");
                for (key, value) in &pairs {
                    cmd.arg(key).arg(value);
                }
                cmd.query_async(&mut conn).await
            }
        })
        .await;

        match result {
            Ok(_) => Ok(Value::String("OK".to_string())),
            Err(e) => Ok(Value::error(format!("Mset error: {}", e))),
        }
    })
}

/// kvs/redis-lrange - リストの範囲を取得
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: リストのキー
/// - start: 開始インデックス（0ベース）
/// - stop: 終了インデックス（-1で最後まで）
///
/// 戻り値: 要素のベクター or {:error message}
pub fn native_redis_lrange(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-lrange", "4"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lrange (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lrange (key)", "strings"],
            ))
        }
    };

    let start = match &args[2] {
        Value::Integer(i) => *i as isize,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lrange (start)", "integers"],
            ))
        }
    };

    let stop = match &args[3] {
        Value::Integer(i) => *i as isize,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-lrange (stop)", "integers"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<Vec<String>> =
            execute_with_retry(url, |mut conn| async move {
                conn.lrange(key, start, stop).await
            })
            .await;

        match result {
            Ok(values) => Ok(Value::Vector(
                values
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(e) => Ok(Value::error(format!("Lrange error: {}", e))),
        }
    })
}
