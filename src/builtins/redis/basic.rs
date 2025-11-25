//! Redis basic operations

use super::connection::{execute_with_retry, TOKIO_RT};
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use redis::AsyncCommands;

/// kvs/redis-get - Redisからキーの値を取得
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: キー名
///
/// 戻り値: 値 or nil or {:error message}
pub fn native_redis_get(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-get");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-get (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-get (key)", "strings"],
            ))
        }
    };

    // 非同期処理を同期的に実行（グローバルランタイムを使用）
    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| async move { conn.get(key).await }).await;

        match result {
            Ok(Some(value)) => Ok(Value::String(value)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Ok(Value::error(format!("Get error: {}", e))),
        }
    })
}

/// kvs/redis-set - キーに値を設定
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - value: 値
///
/// 戻り値: "OK" or {:error message}
pub fn native_redis_set(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "kvs/redis-set");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-set (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-set (key)", "strings"],
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
                    "kvs/redis-set (value)",
                    "strings, integers, floats, or bools",
                ],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<String> = execute_with_retry(url, |mut conn| {
            let value = value.clone();
            async move { conn.set(key, &value).await }
        })
        .await;

        match result {
            Ok(_) => Ok(Value::String("OK".to_string())),
            Err(e) => Ok(Value::error(format!("Set error: {}", e))),
        }
    })
}

/// kvs/redis-delete - キーを削除
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: 削除されたキー数 or {:error message}
pub fn native_redis_delete(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-delete");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-delete (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-delete (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| async move { conn.del(key).await }).await;

        match result {
            Ok(count) => Ok(Value::Integer(count)),
            Err(e) => Ok(Value::error(format!("Delete error: {}", e))),
        }
    })
}

/// kvs/redis-exists? - キーが存在するかチェック
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: true or false or {:error message}
pub fn native_redis_exists(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-exists?");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-exists? (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-exists? (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result =
            execute_with_retry(url, |mut conn| async move { conn.exists(key).await }).await;

        match result {
            Ok(exists) => Ok(Value::Bool(exists)),
            Err(e) => Ok(Value::error(format!("Exists error: {}", e))),
        }
    })
}

/// kvs/redis-keys - パターンにマッチするキー一覧を取得
///
/// 引数:
/// - url: 接続URL
/// - pattern: パターン（例: "user:*"）
///
/// 戻り値: キーのベクタ or {:error message}
pub fn native_redis_keys(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-keys");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-keys (url)", "strings"],
            ))
        }
    };

    let pattern = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-keys (pattern)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result: redis::RedisResult<Vec<String>> =
            execute_with_retry(url, |mut conn| async move { conn.keys(pattern).await }).await;

        match result {
            Ok(keys) => Ok(Value::Vector(
                keys.into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(e) => Ok(Value::error(format!("Keys error: {}", e))),
        }
    })
}

/// kvs/redis-expire - キーに有効期限を設定（秒）
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - seconds: 有効期限（秒）
///
/// 戻り値: true or false or {:error message}
pub fn native_redis_expire(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "kvs/redis-expire");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-expire (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-expire (key)", "strings"],
            ))
        }
    };

    let seconds = match &args[2] {
        Value::Integer(i) if *i >= 0 => *i,
        Value::Integer(_) => {
            return Err(fmt_msg(
                MsgKey::MustBeNonNegative,
                &["kvs/redis-expire", "seconds"],
            ))
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-expire (seconds)", "integers"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(
            url,
            |mut conn| async move { conn.expire(key, seconds).await },
        )
        .await;

        match result {
            Ok(success) => Ok(Value::Bool(success)),
            Err(e) => Ok(Value::error(format!("Expire error: {}", e))),
        }
    })
}

/// kvs/redis-ttl - キーの残り有効期限を取得（秒）
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: 残り秒数 or -1 (期限なし) or -2 (存在しない) or {:error message}
pub fn native_redis_ttl(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "kvs/redis-ttl");

    let url = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-ttl (url)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-ttl (key)", "strings"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| async move { conn.ttl(key).await }).await;

        match result {
            Ok(ttl) => Ok(Value::Integer(ttl)),
            Err(e) => Ok(Value::error(format!("TTL error: {}", e))),
        }
    })
}
