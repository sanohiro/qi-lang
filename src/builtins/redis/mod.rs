//! Redis接続モジュール
//!
//! このモジュールは `kvs-redis` feature でコンパイルされます。

pub mod basic;
pub mod connection;
pub mod counter;
pub mod hash;
pub mod list;
pub mod multi;
pub mod set;

pub use basic::*;
pub use counter::*;
pub use hash::*;
pub use list::*;
pub use multi::*;
pub use set::*;

/// 登録すべき関数のリスト（全21関数）
/// @qi-doc:category kvs/redis
/// @qi-doc:functions kvs/redis-get, kvs/redis-set, kvs/redis-delete, kvs/redis-exists?, kvs/redis-keys, kvs/redis-expire, kvs/redis-ttl, kvs/redis-incr, kvs/redis-decr, kvs/redis-lpush, kvs/redis-rpush, kvs/redis-lpop, kvs/redis-rpop, kvs/redis-lrange, kvs/redis-hset, kvs/redis-hget, kvs/redis-hgetall, kvs/redis-sadd, kvs/redis-smembers, kvs/redis-mget, kvs/redis-mset
pub const FUNCTIONS: super::NativeFunctions = &[
    // 基本操作（7関数）
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
