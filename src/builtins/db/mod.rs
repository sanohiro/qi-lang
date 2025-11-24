//! データベース統一API
//!
//! 複数のデータベース（SQLite, PostgreSQL, MySQL等）に対する統一インターフェースを提供。
//! Phase 1: 基本操作、サニタイズ
//! Phase 2: トランザクション、メタデータAPI、ストアドプロシージャ/ファンクション
//! Phase 3: コネクションプーリング

use crate::value::Value;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub mod advanced;
pub mod connection;
pub mod helpers;
pub mod metadata;
pub mod pool;
pub mod pool_ops;
pub mod query;
pub mod sanitize;
pub mod traits;
pub mod transaction;
pub mod types;

pub use advanced::*;
pub use connection::*;
pub use helpers::*;
pub use metadata::*;
pub use pool::*;
pub use pool_ops::*;
pub use query::*;
pub use sanitize::*;
pub use traits::*;
pub use transaction::*;
pub use types::*;

/// グローバル接続マネージャー
pub(super) static CONNECTIONS: LazyLock<Mutex<HashMap<String, Arc<dyn DbConnection>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub(super) static NEXT_CONN_ID: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

/// グローバルトランザクションマネージャー
pub(super) static TRANSACTIONS: LazyLock<Mutex<HashMap<String, Arc<dyn DbTransaction>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub(super) static NEXT_TX_ID: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

/// グローバルプールマネージャー
pub(super) static POOLS: LazyLock<Mutex<HashMap<String, DbPool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub(super) static NEXT_POOL_ID: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

/// 接続IDを生成
pub(super) fn gen_conn_id() -> String {
    let mut id = NEXT_CONN_ID.lock();
    let conn_id = format!("conn_{}", *id);
    *id += 1;
    conn_id
}

/// 接続IDを抽出
pub(super) fn extract_conn_id(value: &Value) -> Result<String, String> {
    use crate::i18n::{fmt_msg, MsgKey};
    const DB_CONN_PREFIX: &str = "DbConnection:";
    match value {
        Value::String(s) if s.starts_with(DB_CONN_PREFIX) => Ok(s
            .strip_prefix(DB_CONN_PREFIX)
            .expect("checked by starts_with above")
            .to_string()),
        _ => Err(fmt_msg(
            MsgKey::DbExpectedConnection,
            &[&format!("{:?}", value)],
        )),
    }
}

/// トランザクションIDを生成
pub(super) fn gen_tx_id() -> String {
    let mut id = NEXT_TX_ID.lock();
    let tx_id = format!("tx_{}", *id);
    *id += 1;
    tx_id
}

/// トランザクションIDを抽出
pub(super) fn extract_tx_id(value: &Value) -> Result<String, String> {
    use crate::i18n::{fmt_msg, MsgKey};
    const DB_TX_PREFIX: &str = "DbTransaction:";
    match value {
        Value::String(s) if s.starts_with(DB_TX_PREFIX) => Ok(s
            .strip_prefix(DB_TX_PREFIX)
            .expect("checked by starts_with above")
            .to_string()),
        _ => Err(fmt_msg(
            MsgKey::DbExpectedTransaction,
            &[&format!("{:?}", value)],
        )),
    }
}

/// プールIDを生成
pub(super) fn gen_pool_id() -> String {
    let mut id = NEXT_POOL_ID.lock();
    let pool_id = format!("pool_{}", *id);
    *id += 1;
    pool_id
}

/// プールIDを抽出
pub(super) fn extract_pool_id(value: &Value) -> Result<String, String> {
    use crate::i18n::{fmt_msg, MsgKey};
    const DB_POOL_PREFIX: &str = "DbPool:";
    match value {
        Value::String(s) if s.starts_with(DB_POOL_PREFIX) => Ok(s
            .strip_prefix(DB_POOL_PREFIX)
            .expect("checked by starts_with above")
            .to_string()),
        _ => Err(fmt_msg(MsgKey::DbExpectedPool, &[&format!("{:?}", value)])),
    }
}

/// 接続またはトランザクションを判別
pub(super) enum ConnOrTx {
    Conn(String),
    Tx(String),
}

/// 接続IDまたはトランザクションIDを抽出
pub(super) fn extract_conn_or_tx(value: &Value) -> Result<ConnOrTx, String> {
    use crate::i18n::{fmt_msg, MsgKey};
    const DB_CONN_PREFIX: &str = "DbConnection:";
    const DB_TX_PREFIX: &str = "DbTransaction:";
    match value {
        Value::String(s) if s.starts_with(DB_CONN_PREFIX) => Ok(ConnOrTx::Conn(
            s.strip_prefix(DB_CONN_PREFIX)
                .expect("checked by starts_with above")
                .to_string(),
        )),
        Value::String(s) if s.starts_with(DB_TX_PREFIX) => Ok(ConnOrTx::Tx(
            s.strip_prefix(DB_TX_PREFIX)
                .expect("checked by starts_with above")
                .to_string(),
        )),
        _ => Err(fmt_msg(
            MsgKey::DbExpectedConnectionOrTransaction,
            &[&format!("{:?}", value)],
        )),
    }
}

/// 登録すべき関数のリスト
/// @qi-doc:category database
/// @qi-doc:functions db/connect, db/query, db/query-one, db/exec, db/close, db/sanitize, db/sanitize-id, db/escape-like, db/begin, db/commit, db/rollback, db/tables, db/columns, db/indexes, db/foreign-keys, db/call, db/supports?, db/driver-info, db/query-info
pub const FUNCTIONS: super::NativeFunctions = &[
    ("db/connect", native_connect),
    ("db/query", native_query),
    ("db/query-one", native_query_one),
    ("db/exec", native_exec),
    ("db/close", native_close),
    ("db/sanitize", native_sanitize),
    ("db/sanitize-id", native_sanitize_identifier),
    ("db/escape-like", native_escape_like),
    ("db/begin", native_begin),
    ("db/commit", native_commit),
    ("db/rollback", native_rollback),
    ("db/tables", native_tables),
    ("db/columns", native_columns),
    ("db/indexes", native_indexes),
    ("db/foreign-keys", native_foreign_keys),
    ("db/call", native_call),
    ("db/supports?", native_supports),
    ("db/driver-info", native_driver_info),
    ("db/query-info", native_query_info),
    ("db/create-pool", native_create_pool),
    ("db/pool-acquire", native_pool_acquire),
    ("db/pool-release", native_pool_release),
    ("db/pool-close", native_pool_close),
    ("db/pool-stats", native_pool_stats),
];
