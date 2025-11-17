use super::*;

/// Redis KVSドライバー
#[cfg(feature = "kvs-redis")]
pub(super) struct RedisDriver {
    url: String,
}

#[cfg(feature = "kvs-redis")]
impl RedisDriver {
    pub(super) fn new(url: &str) -> Self {
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn delete(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_delete(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn exists(&self, key: &str) -> Result<bool, String> {
        crate::builtins::redis::native_redis_exists(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Bool(b) => Ok(b),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn ttl(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_ttl(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn incr(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_incr(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn decr(&self, key: &str) -> Result<i64, String> {
        crate::builtins::redis::native_redis_decr(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        .and_then(|v| match v {
            Value::Integer(i) => Ok(i),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn mget(&self, keys: &[String]) -> Result<Vec<Option<String>>, String> {
        let keys_vec: Vec<Value> = keys.iter().map(|k| Value::String(k.clone())).collect();
        crate::builtins::redis::native_redis_mget(&[
            Value::String(self.url.clone()),
            Value::Vector(keys_vec.into()),
        ])
        .and_then(|v| match v {
            Value::Vector(vec) => Ok(vec
                .iter()
                .map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Nil => None,
                    _ => None,
                })
                .collect()),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn mset(&self, pairs: &HashMap<String, String>) -> Result<String, String> {
        let mut map = crate::new_hashmap();
        for (k, v) in pairs {
            map.insert(k.clone(), Value::String(v.clone()));
        }
        crate::builtins::redis::native_redis_mset(&[
            Value::String(self.url.clone()),
            Value::Map(map),
        ])
        .and_then(|v| match v {
            Value::String(s) => Ok(s),
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }

    fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, String> {
        crate::builtins::redis::native_redis_lrange(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
            Value::Integer(start),
            Value::Integer(stop),
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
            Value::Map(m) if m.contains_key(":error") => {
                // SAFETY: contains_keyでチェック済み
                Err(m.get(":error").expect(":error key exists").to_string())
            }
            _ => Err(fmt_msg(MsgKey::UnexpectedResponse, &[])),
        })
    }
}
