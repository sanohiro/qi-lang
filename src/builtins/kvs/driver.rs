use super::*;

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
