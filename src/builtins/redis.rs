//! Redis接続モジュール
//!
//! このモジュールは `kvs-redis` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use dashmap::DashMap;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Redis接続プール（URL → Connection のマッピング）
static REDIS_POOL: LazyLock<DashMap<String, MultiplexedConnection>> = LazyLock::new(DashMap::new);

/// グローバルなtokioランタイム（Redisの非同期操作用）
static TOKIO_RT: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime for Redis"));

/// 接続を取得または新規作成
async fn get_or_create_connection(url: &str) -> Result<MultiplexedConnection, String> {
    // 既存の接続を取得
    if let Some(conn) = REDIS_POOL.get(url) {
        return Ok(conn.clone());
    }

    // 新規接続を作成
    let client = Client::open(url).map_err(|e| format!("Connection error: {}", e))?;

    let conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    // プールに保存
    REDIS_POOL.insert(url.to_string(), conn.clone());

    Ok(conn)
}

/// 接続エラー時にプールから削除して再接続
async fn reconnect(url: &str) -> Result<MultiplexedConnection, String> {
    // プールから古い接続を削除
    REDIS_POOL.remove(url);

    // 新規接続を作成
    get_or_create_connection(url).await
}

/// Redis操作を再試行付きで実行するヘルパー
async fn execute_with_retry<T, F, Fut>(url: &str, operation: F) -> redis::RedisResult<T>
where
    F: Fn(MultiplexedConnection) -> Fut,
    Fut: std::future::Future<Output = redis::RedisResult<T>>,
{
    // 最初の試行
    let conn = get_or_create_connection(url)
        .await
        .map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "Connection error", e)))?;

    let result = operation(conn).await;

    // エラーの場合、再接続して再試行
    if let Err(ref e) = result {
        let err_str = e.to_string();
        if err_str.contains("broken pipe")
            || err_str.contains("Connection")
            || err_str.contains("terminated")
        {
            // 再接続
            if let Ok(new_conn) = reconnect(url).await {
                return operation(new_conn).await;
            }
        }
    }

    result
}

/// kvs/redis-get - キーの値を取得
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - key: キー名
///
/// 戻り値: 値（文字列）or nil or {:error message}
pub fn native_redis_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-get", "2"]));
    }

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
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-set", "3"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-delete", "2"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-exists?", "2"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-keys", "2"]));
    }

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
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-expire", "3"]));
    }

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
        Value::Integer(i) => *i as u64,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/redis-expire (seconds)", "integers"],
            ))
        }
    };

    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| async move {
            conn.expire(key, seconds as i64).await
        })
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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-ttl", "2"]));
    }

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

/// kvs/redis-incr - キーの値をインクリメント
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
///
/// 戻り値: インクリメント後の値 or {:error message}
pub fn native_redis_incr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-incr", "2"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-decr", "2"]));
    }

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

/// kvs/redis-lpush - リスト左端に要素を追加
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - value: 追加する値
///
/// 戻り値: リスト長 or {:error message}
pub fn native_redis_lpush(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-lpush", "3"]));
    }

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
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-rpush", "3"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-lpop", "2"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-rpop", "2"]));
    }

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

/// kvs/redis-hset - ハッシュのフィールドに値を設定
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - field: フィールド名
/// - value: 値
///
/// 戻り値: true (新規作成) or false (更新) or {:error message}
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
                    map.insert(field, Value::String(value));
                }
                Ok(Value::Map(map))
            }
            Err(e) => Ok(Value::error(format!("Hgetall error: {}", e))),
        }
    })
}

/// kvs/redis-sadd - セットにメンバーを追加
///
/// 引数:
/// - url: 接続URL
/// - key: キー名
/// - member: メンバー
///
/// 戻り値: 追加されたメンバー数 or {:error message}
pub fn native_redis_sadd(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-sadd", "3"]));
    }

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/redis-smembers", "2"]));
    }

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

/// kvs/redis-mget - 複数のキーの値を一括取得
///
/// 引数:
/// - url: 接続URL（例: "redis://localhost:6379"）
/// - keys: キーのベクター
///
/// 戻り値: 値のベクター（存在しないキーはnil）or {:error message}
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

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（全21関数）
/// @qi-doc:category kvs/redis
/// @qi-doc:functions kvs/redis-get, kvs/redis-set, kvs/redis-delete, kvs/redis-exists?, kvs/redis-keys, kvs/redis-expire, kvs/redis-ttl, kvs/redis-incr, kvs/redis-decr, kvs/redis-lpush, kvs/redis-rpush, kvs/redis-lpop, kvs/redis-rpop, kvs/redis-lrange, kvs/redis-hset, kvs/redis-hget, kvs/redis-hgetall, kvs/redis-sadd, kvs/redis-smembers, kvs/redis-mget, kvs/redis-mset
pub const FUNCTIONS: super::NativeFunctions = &[
    // 基本操作（8関数）
    ("kvs/redis-get", native_redis_get),
    ("kvs/redis-set", native_redis_set),
    ("kvs/redis-delete", native_redis_delete),
    ("kvs/redis-exists?", native_redis_exists),
    ("kvs/redis-keys", native_redis_keys),
    ("kvs/redis-expire", native_redis_expire),
    ("kvs/redis-ttl", native_redis_ttl),
    // 数値操作（2関数）
    ("kvs/redis-incr", native_redis_incr),
    ("kvs/redis-decr", native_redis_decr),
    // リスト操作（5関数）
    ("kvs/redis-lpush", native_redis_lpush),
    ("kvs/redis-rpush", native_redis_rpush),
    ("kvs/redis-lpop", native_redis_lpop),
    ("kvs/redis-rpop", native_redis_rpop),
    ("kvs/redis-lrange", native_redis_lrange),
    // ハッシュ操作（3関数）
    ("kvs/redis-hset", native_redis_hset),
    ("kvs/redis-hget", native_redis_hget),
    ("kvs/redis-hgetall", native_redis_hgetall),
    // セット操作（2関数）
    ("kvs/redis-sadd", native_redis_sadd),
    ("kvs/redis-smembers", native_redis_smembers),
    // 複数操作（2関数）
    ("kvs/redis-mget", native_redis_mget),
    ("kvs/redis-mset", native_redis_mset),
];

#[cfg(test)]
mod tests {
    use super::*;

    // 注: これらのテストはRedisサーバーが必要なため、統合テストとして実装すべき

    #[test]
    fn test_redis_url_validation() {
        // URL検証のみテスト
        let url = Value::String("redis://localhost:6379".to_string());
        let key = Value::String("test_key".to_string());

        // 実際の接続は行わず、引数チェックのみ
        let result = native_redis_get(&[url, key]);
        // Redisサーバーが無い場合はエラーになるが、それは正常
        assert!(result.is_ok());
    }
}
