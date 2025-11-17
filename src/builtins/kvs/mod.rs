//! KVS統一API
//!
//! 複数のKVSバックエンド（Redis, Memcached等）に対する統一インターフェースを提供。
//! データベースAPIと同様の設計（接続IDをStringで管理）。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;

mod driver;
mod connection;
mod basic;
mod counter;
mod list;
mod hash;
mod set;
mod multi;

#[cfg(feature = "kvs-redis")]
mod redis_driver;

pub use driver::*;
pub use connection::*;
pub use basic::*;
pub use counter::*;
pub use list::*;
pub use hash::*;
pub use set::*;
pub use multi::*;

/// KVS接続プール（接続ID → KvsDriver のマッピング）
pub(super) static CONNECTIONS: LazyLock<Mutex<HashMap<String, Box<dyn KvsDriver>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 接続ID生成カウンター
static CONN_COUNTER: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

/// 接続IDを生成
pub(super) fn gen_conn_id() -> String {
    let mut counter = CONN_COUNTER.lock();
    *counter += 1;
    format!("kvs:{}", *counter)
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
