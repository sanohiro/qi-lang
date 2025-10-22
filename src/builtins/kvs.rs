//! KVS統一API
//!
//! 複数のKVSバックエンド（Redis, Memcached等）に対する統一インターフェースを提供。
//! データベースAPIと同様の設計（接続IDをStringで管理）。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// KVS接続プール（接続ID → KvsDriver のマッピング）
static CONNECTIONS: LazyLock<Mutex<HashMap<String, Box<dyn KvsDriver>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 接続ID生成カウンター
static CONN_COUNTER: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

/// 接続IDを生成
fn gen_conn_id() -> String {
    let mut counter = CONN_COUNTER.lock();
    *counter += 1;
    format!("kvs:{}", *counter)
}

/// KVSドライバートレイト（統一インターフェース）
pub trait KvsDriver: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&self, key: &str, value: &str) -> Result<String, String>;
    fn delete(&self, key: &str) -> Result<i64, String>;
    fn exists(&self, key: &str) -> Result<bool, String>;
    fn keys(&self, pattern: &str) -> Result<Vec<String>, String>;
    fn expire(&self, key: &str, seconds: i64) -> Result<bool, String>;
    fn ttl(&self, key: &str) -> Result<i64, String>;
    fn incr(&self, key: &str) -> Result<i64, String>;
    fn decr(&self, key: &str) -> Result<i64, String>;
    fn lpush(&self, key: &str, value: &str) -> Result<i64, String>;
    fn rpush(&self, key: &str, value: &str) -> Result<i64, String>;
    fn lpop(&self, key: &str) -> Result<Option<String>, String>;
    fn rpop(&self, key: &str) -> Result<Option<String>, String>;
    fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool, String>;
    fn hget(&self, key: &str, field: &str) -> Result<Option<String>, String>;
    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>, String>;
    fn sadd(&self, key: &str, member: &str) -> Result<i64, String>;
    fn smembers(&self, key: &str) -> Result<Vec<String>, String>;

    // 複数操作（一部のKVSでのみサポート）
    fn mget(&self, keys: &[String]) -> Result<Vec<Option<String>>, String>;
    fn mset(&self, pairs: &HashMap<String, String>) -> Result<String, String>;

    // リスト操作（一部のKVSでのみサポート）
    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, String>;
}

/// Redis KVSドライバー
#[cfg(feature = "kvs-redis")]
struct RedisDriver {
    url: String,
}

#[cfg(feature = "kvs-redis")]
impl RedisDriver {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

#[cfg(feature = "kvs-redis")]
impl KvsDriver for RedisDriver {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        crate::builtins::redis::native_redis_get(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Nil => Ok(None),
            Value::String(s) => Ok(Some(s)),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn set(&self, key: &str, value: &str) -> Result<String, String> {
        crate::builtins::redis::native_redis_set(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        ])
        .and_then(|v| match v {
            Value::String(s) => Ok(s),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn delete(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_delete(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn exists(&self, key: &str) -> Result<bool, String> {
        crate::builtins::redis::native_redis_exists(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Bool(b) => Ok(b),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn keys(&self, pattern: &str) -> Result<Vec<String>, String> {
        crate::builtins::redis::native_redis_keys(&[
            Value::String(self.url.clone()),
            Value::String(pattern.to_string()),
        ])
        .and_then(|v| match v {
            Value::Vector(vec) => Ok(vec
                .iter()
                .filter_map(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect()),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn expire(&self, key: &str, seconds: i64) -> Result<bool, String> {
        crate::builtins::redis::native_redis_expire(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::Integer(seconds),
        ])
        .and_then(|v| match v {
            Value::Bool(b) => Ok(b),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn ttl(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_ttl(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn incr(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_incr(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn decr(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_decr(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn lpush(&self, key: &str, value: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_lpush(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn rpush(&self, key: &str, value: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_rpush(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn lpop(&self, key: &str) -> Result<Option<String>, String> {
        crate::builtins::redis::native_redis_lpop(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Nil => Ok(None),
            Value::String(s) => Ok(Some(s)),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn rpop(&self, key: &str) -> Result<Option<String>, String> {
        crate::builtins::redis::native_redis_rpop(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Nil => Ok(None),
            Value::String(s) => Ok(Some(s)),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool, String> {
        crate::builtins::redis::native_redis_hset(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(field.to_string()),
            Value::String(value.to_string()),
        ])
        .and_then(|v| match v {
            Value::Bool(b) => Ok(b),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn hget(&self, key: &str, field: &str) -> Result<Option<String>, String> {
        crate::builtins::redis::native_redis_hget(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(field.to_string()),
        ])
        .and_then(|v| match v {
            Value::Nil => Ok(None),
            Value::String(s) => Ok(Some(s)),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>, String> {
        crate::builtins::redis::native_redis_hgetall(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Map(m) => {
                if m.contains_key(":error") {
                    return Err(m.get(":error").unwrap().to_string());
                }
                let mut pairs = Vec::new();
                for (k, v) in m.iter() {
                    if let Value::String(s) = v {
                        // キーワード形式 ":field" から "field" に変換
                        let field = if let Some(stripped) = k.strip_prefix(':') {
                            stripped.to_string()
                        } else {
                            k.clone()
                        };
                        pairs.push((field, s.clone()));
                    }
                }
                Ok(pairs)
            }
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn sadd(&self, key: &str, member: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_sadd(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::String(member.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn smembers(&self, key: &str) -> Result<Vec<String>, String> {
        crate::builtins::redis::native_redis_smembers(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Vector(vec) => Ok(vec
                .iter()
                .filter_map(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect()),
            Value::Map(m) if m.contains_key(":error") => Err(m.get(":error").unwrap().to_string()),
            _ => Err("Unexpected response".to_string()),
        })
    }

    fn mget(&self, keys: &[String]) -> Result<Vec<Option<String>>, String> {
        // TODO: redis.rsにnative_redis_mgetを実装する必要がある
        Err("MGET not yet implemented for Redis".to_string())
    }

    fn mset(&self, pairs: &HashMap<String, String>) -> Result<String, String> {
        // TODO: redis.rsにnative_redis_msetを実装する必要がある
        Err("MSET not yet implemented for Redis".to_string())
    }

    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, String> {
        // TODO: redis.rsにnative_redis_lrangeを実装する必要がある
        Err("LRANGE not yet implemented for Redis".to_string())
    }
}

// ========================================
// 統一インターフェース関数
// ========================================

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
            return Err("Redis support not enabled (feature 'kvs-redis' required)".to_string());
        }
    } else {
        return Err(format!("Unsupported KVS URL: {}", url));
    };

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), driver);

    Ok(Value::String(format!("KvsConnection:{}", conn_id)))
}

/// 接続IDから接続を取得
fn get_connection(conn_str: &str) -> Result<String, String> {
    if !conn_str.starts_with("KvsConnection:") {
        return Err("Invalid connection (expected KvsConnection:xxx)".to_string());
    }
    let conn_id = &conn_str["KvsConnection:".len()..];
    if !CONNECTIONS.lock().contains_key(conn_id) {
        return Err(format!("Connection not found: {}", conn_id));
    }
    Ok(conn_id.to_string())
}

/// kvs/get - キーの値を取得
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/get", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/get (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/get (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.get(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/set - キーに値を設定
pub fn native_set(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/set", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/set (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/set (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/set (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.set(key, &value) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/delete - キーを削除
pub fn native_delete(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/delete", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/delete (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/delete (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.delete(key) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/exists? - キーが存在するかチェック
pub fn native_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/exists?", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/exists? (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/exists? (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.exists(key) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/keys - パターンにマッチするキー一覧を取得
pub fn native_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/keys", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/keys (conn)", "strings"])),
    };

    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/keys (pattern)", "strings"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.keys(pattern) {
        Ok(keys) => Ok(Value::Vector(
            keys.into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/expire - キーに有効期限を設定（秒）
pub fn native_expire(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/expire", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/expire (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/expire (key)", "strings"])),
    };

    let seconds = match &args[2] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/expire (seconds)", "integers"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.expire(key, seconds) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/ttl - キーの残り有効期限を取得（秒）
pub fn native_ttl(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/ttl", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/ttl (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/ttl (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.ttl(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/incr - キーの値をインクリメント
pub fn native_incr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/incr", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/incr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.incr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/decr - キーの値をデクリメント
pub fn native_decr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/decr", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/decr (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.decr(key) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/lpush - リスト左端に要素を追加
pub fn native_lpush(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lpush", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpush (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpush (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/lpush (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.lpush(key, &value) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/rpush - リスト右端に要素を追加
pub fn native_rpush(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/rpush", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpush (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpush (key)", "strings"])),
    };

    let value = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/rpush (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.rpush(key, &value) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/lpop - リスト左端から要素を取得
pub fn native_lpop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lpop", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpop (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lpop (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.lpop(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/rpop - リスト右端から要素を取得
pub fn native_rpop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/rpop", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpop (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/rpop (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.rpop(key) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/hset - ハッシュのフィールドに値を設定
pub fn native_hset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hset", "4"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (key)", "strings"])),
    };

    let field = match &args[2] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hset (field)", "strings"])),
    };

    let value = match &args[3] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/hset (value)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.hset(key, field, &value) {
        Ok(b) => Ok(Value::Bool(b)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/hget - ハッシュのフィールドから値を取得
pub fn native_hget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hget", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (key)", "strings"])),
    };

    let field = match &args[2] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hget (field)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.hget(key, field) {
        Ok(Some(v)) => Ok(Value::String(v)),
        Ok(None) => Ok(Value::Nil),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/hgetall - ハッシュ全体を取得
pub fn native_hgetall(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/hgetall", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/hgetall (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/hgetall (key)", "strings"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.hgetall(key) {
        Ok(pairs) => {
            let mut map = im::HashMap::new();
            for (field, value) in pairs {
                // キーワード形式（:field）でマップに追加
                map.insert(format!(":{}", field), Value::String(value));
            }
            Ok(Value::Map(map))
        }
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/sadd - セットにメンバーを追加
pub fn native_sadd(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/sadd", "3"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/sadd (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/sadd (key)", "strings"])),
    };

    let member = match &args[2] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/sadd (member)", "strings, integers, floats, or bools"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.sadd(key, &member) {
        Ok(i) => Ok(Value::Integer(i)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/smembers - セットの全メンバーを取得
pub fn native_smembers(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/smembers", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/smembers (conn)", "strings"],
            ))
        }
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["kvs/smembers (key)", "strings"],
            ))
        }
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.smembers(key) {
        Ok(members) => Ok(Value::Vector(
            members
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/mget - 複数のキーの値を一括取得
pub fn native_mget(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/mget", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mget (conn)", "strings"])),
    };

    let keys = match &args[1] {
        Value::Vector(v) => v
            .iter()
            .map(|k| match k {
                Value::String(s) => Ok(s.clone()),
                _ => Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["kvs/mget (keys)", "vector of strings"],
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mget (keys)", "vectors"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.mget(&keys) {
        Ok(values) => Ok(Value::Vector(
            values
                .into_iter()
                .map(|opt| match opt {
                    Some(s) => Value::String(s),
                    None => Value::Nil,
                })
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/mset - 複数のキーと値を一括設定
pub fn native_mset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/mset", "2"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mset (conn)", "strings"])),
    };

    let pairs = match &args[1] {
        Value::Map(m) => {
            let mut map = HashMap::new();
            for (k, v) in m.iter() {
                let value_str = match v {
                    Value::String(s) => s.clone(),
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["kvs/mset (values)", "strings, integers, floats, or bools"],
                        ))
                    }
                };
                map.insert(k.clone(), value_str);
            }
            map
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/mset (pairs)", "maps"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.mset(&pairs) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(e)),
    }
}

/// kvs/lrange - リストの範囲を取得
pub fn native_lrange(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["kvs/lrange", "4"]));
    }

    let conn_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (conn)", "strings"])),
    };

    let key = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (key)", "strings"])),
    };

    let start = match &args[2] {
        Value::Integer(i) => *i,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (start)", "integers"])),
    };

    let stop = match &args[3] {
        Value::Integer(i) => *i,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["kvs/lrange (stop)", "integers"])),
    };

    let conn_id = get_connection(conn_str)?;
    let connections = CONNECTIONS.lock();
    let driver = connections
        .get(&conn_id)
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    match driver.lrange(key, start, stop) {
        Ok(items) => Ok(Value::Vector(
            items
                .into_iter()
                .map(Value::String)
                .collect::<Vec<_>>()
                .into(),
        )),
        Err(e) => Ok(Value::error(e)),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（全22関数）
/// @qi-doc:category kvs
/// @qi-doc:functions kvs/connect, kvs/get, kvs/set, kvs/del, kvs/exists, kvs/keys, kvs/expire, kvs/ttl, kvs/incr, kvs/decr, kvs/lpush, kvs/rpush, kvs/lpop, kvs/rpop, kvs/lrange, kvs/hset, kvs/hget, kvs/hgetall, kvs/sadd, kvs/smembers, kvs/mget, kvs/mset
pub const FUNCTIONS: super::NativeFunctions = &[
    // 接続
    ("kvs/connect", native_connect),
    // 基本操作（7関数）
    ("kvs/get", native_get),
    ("kvs/set", native_set),
    ("kvs/del", native_delete),
    ("kvs/exists", native_exists),
    ("kvs/keys", native_keys),
    ("kvs/expire", native_expire),
    ("kvs/ttl", native_ttl),
    // 数値操作（2関数）
    ("kvs/incr", native_incr),
    ("kvs/decr", native_decr),
    // リスト操作（5関数）
    ("kvs/lpush", native_lpush),
    ("kvs/rpush", native_rpush),
    ("kvs/lpop", native_lpop),
    ("kvs/rpop", native_rpop),
    ("kvs/lrange", native_lrange),
    // ハッシュ操作（3関数）
    ("kvs/hset", native_hset),
    ("kvs/hget", native_hget),
    ("kvs/hgetall", native_hgetall),
    // セット操作（2関数）
    ("kvs/sadd", native_sadd),
    ("kvs/smembers", native_smembers),
    // 複数操作（2関数）
    ("kvs/mget", native_mget),
    ("kvs/mset", native_mset),
];
