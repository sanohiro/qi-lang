//! データベース統一API
//!
//! 複数のデータベース（SQLite, PostgreSQL, MySQL等）に対する統一インターフェースを提供。
//! Phase 1: 基本操作、サニタイズ
//! Phase 2: トランザクション、メタデータAPI、ストアドプロシージャ/ファンクション
//! Phase 3: コネクションプーリング

use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// データベースエラー
#[derive(Debug, Clone)]
pub struct DbError {
    pub message: String,
}

impl DbError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl From<String> for DbError {
    fn from(s: String) -> Self {
        DbError::new(s)
    }
}

impl From<&str> for DbError {
    fn from(s: &str) -> Self {
        DbError::new(s)
    }
}

pub type DbResult<T> = Result<T, DbError>;

/// データベース行（カラム名 -> 値のマップ）
pub type Row = HashMap<String, Value>;

/// クエリ結果（行のベクター）
pub type Rows = Vec<Row>;

/// データベース接続オプション
#[derive(Debug, Clone)]
pub struct ConnectionOptions {
    pub timeout_ms: Option<u64>,
    pub read_only: bool,
    pub auto_commit: bool,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000), // 30秒
            read_only: false,
            auto_commit: true,
        }
    }
}

impl ConnectionOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            if let Some(Value::Integer(ms)) = map.get("timeout") {
                options.timeout_ms = Some(*ms as u64);
            }
            if let Some(Value::Bool(ro)) = map.get("read-only") {
                options.read_only = *ro;
            }
            if let Some(Value::Bool(ac)) = map.get("auto-commit") {
                options.auto_commit = *ac;
            }
        }

        Ok(options)
    }
}

/// クエリオプション
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub timeout_ms: Option<u64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(5000), // 5秒
            limit: None,
            offset: None,
        }
    }
}

impl QueryOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            if let Some(Value::Integer(ms)) = map.get("timeout") {
                options.timeout_ms = Some(*ms as u64);
            }
            if let Some(Value::Integer(n)) = map.get("limit") {
                options.limit = Some(*n);
            }
            if let Some(Value::Integer(n)) = map.get("offset") {
                options.offset = Some(*n);
            }
        }

        Ok(options)
    }
}

/// トランザクション分離レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read-uncommitted" => Some(Self::ReadUncommitted),
            "read-committed" => Some(Self::ReadCommitted),
            "repeatable-read" => Some(Self::RepeatableRead),
            "serializable" => Some(Self::Serializable),
            _ => None,
        }
    }
}

/// トランザクションオプション
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub isolation: IsolationLevel,
    pub timeout_ms: Option<u64>,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation: IsolationLevel::ReadCommitted,
            timeout_ms: Some(10000), // 10秒
        }
    }
}

impl TransactionOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            if let Some(Value::String(iso)) = map.get("isolation") {
                options.isolation = IsolationLevel::from_str(iso)
                    .ok_or_else(|| DbError::new(format!("Invalid isolation level: {}", iso)))?;
            }
            if let Some(Value::Integer(ms)) = map.get("timeout") {
                options.timeout_ms = Some(*ms as u64);
            }
        }

        Ok(options)
    }
}

/// データベースドライバーの統一インターフェース
pub trait DbDriver: Send + Sync {
    /// 接続URLからデータベースに接続
    fn connect(&self, url: &str, opts: &ConnectionOptions) -> DbResult<Arc<dyn DbConnection>>;

    /// ドライバー名を取得
    fn name(&self) -> &str;
}

/// データベース接続の統一インターフェース
pub trait DbConnection: Send + Sync {
    /// SQLクエリを実行して結果を取得
    fn query(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Rows>;

    /// SQLクエリを実行して最初の1行のみ取得
    fn query_one(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Option<Row>> {
        let rows = self.query(sql, params, opts)?;
        Ok(rows.into_iter().next())
    }

    /// SQL文を実行して影響を受けた行数を返す
    fn exec(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<i64>;

    /// トランザクションを開始
    fn begin(&self, opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>>;

    /// 接続を閉じる
    fn close(&self) -> DbResult<()>;

    /// サニタイズ: 値をエスケープ（シングルクォート対応）
    fn sanitize(&self, value: &str) -> String;

    /// サニタイズ: 識別子をエスケープ（テーブル名、カラム名）
    fn sanitize_identifier(&self, name: &str) -> String;

    /// LIKE句のパターンをエスケープ
    fn escape_like(&self, pattern: &str) -> String;

    /// ドライバー名を取得
    fn driver_name(&self) -> &str;

    // TODO: Phase 2で実装予定
    // fn call(&self, name: &str, params: &[Value], opts: &CallOptions) -> DbResult<CallResult>;
    // fn tables(&self) -> DbResult<Vec<String>>;
    // fn columns(&self, table: &str) -> DbResult<Vec<ColumnInfo>>;
    // fn supports(&self, feature: &str) -> bool;
}

/// データベーストランザクションの統一インターフェース
pub trait DbTransaction: Send + Sync {
    /// SQLクエリを実行して結果を取得
    fn query(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Rows>;

    /// SQLクエリを実行して最初の1行のみ取得
    fn query_one(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Option<Row>> {
        let rows = self.query(sql, params, opts)?;
        Ok(rows.into_iter().next())
    }

    /// SQL文を実行して影響を受けた行数を返す
    fn exec(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<i64>;

    /// トランザクションをコミット
    fn commit(self: Arc<Self>) -> DbResult<()>;

    /// トランザクションをロールバック
    fn rollback(self: Arc<Self>) -> DbResult<()>;
}

/// Qi Valueに変換
impl From<DbError> for Value {
    fn from(err: DbError) -> Self {
        Value::String(format!("DbError: {}", err.message))
    }
}

/// 行をQi ValueのMapに変換
pub fn row_to_value(row: Row) -> Value {
    Value::Map(row)
}

/// 複数行をQi ValueのVectorに変換
pub fn rows_to_value(rows: Rows) -> Value {
    Value::Vector(rows.into_iter().map(row_to_value).collect())
}

/// Qi Valueからパラメータを抽出
pub fn params_from_value(params: &Value) -> DbResult<Vec<Value>> {
    match params {
        Value::Vector(vec) => Ok(vec.clone()),
        Value::Nil => Ok(vec![]),
        _ => Err(DbError::new("Parameters must be a vector")),
    }
}

// ========================================
// Qi Builtin関数
// ========================================

use super::sqlite::SqliteDriver;
use crate::i18n::fmt_msg;
use crate::i18n::MsgKey;
use parking_lot::Mutex;

lazy_static::lazy_static! {
    /// グローバル接続マネージャー
    static ref CONNECTIONS: Mutex<HashMap<String, Arc<dyn DbConnection>>> = Mutex::new(HashMap::new());
    static ref NEXT_CONN_ID: Mutex<usize> = Mutex::new(0);

    /// グローバルトランザクションマネージャー
    static ref TRANSACTIONS: Mutex<HashMap<String, Arc<dyn DbTransaction>>> = Mutex::new(HashMap::new());
    static ref NEXT_TX_ID: Mutex<usize> = Mutex::new(0);
}

/// 接続IDを生成
fn gen_conn_id() -> String {
    let mut id = NEXT_CONN_ID.lock();
    let conn_id = format!("conn_{}", *id);
    *id += 1;
    conn_id
}

/// db/connect - データベースに接続
pub fn native_connect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["db/connect"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["db/connect", "string"])),
    };

    let opts = if args.len() == 2 {
        ConnectionOptions::from_value(&args[1])
            .map_err(|e| e.message)?
    } else {
        ConnectionOptions::default()
    };

    // URLからドライバーを判定
    let driver: Box<dyn DbDriver> = if url.starts_with("sqlite:") {
        Box::new(SqliteDriver::new())
    } else {
        return Err(fmt_msg(MsgKey::DbUnsupportedUrl, &[url]));
    };

    let conn = driver.connect(url, &opts)
        .map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/query - SQLクエリを実行
pub fn native_query(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(MsgKey::DbNeed2To4Args, &["db/query", &args.len().to_string()]));
    }

    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/query", "string"])),
    };

    let params = if args.len() >= 3 {
        params_from_value(&args[2]).map_err(|e| e.message)?
    } else {
        vec![]
    };

    let opts = if args.len() == 4 {
        QueryOptions::from_value(&args[3]).map_err(|e| e.message)?
    } else {
        QueryOptions::default()
    };

    // 接続かトランザクションかを判別
    let rows = match extract_conn_or_tx(&args[0])? {
        ConnOrTx::Conn(conn_id) => {
            let connections = CONNECTIONS.lock();
            let conn = connections.get(&conn_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;
            conn.query(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            let transactions = TRANSACTIONS.lock();
            let tx = transactions.get(&tx_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;
            tx.query(sql, &params, &opts).map_err(|e| e.message)?
        }
    };

    Ok(rows_to_value(rows))
}

/// db/query-one - SQLクエリを実行して最初の1行のみ取得
pub fn native_query_one(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(MsgKey::DbNeed2To4Args, &["db/query-one", &args.len().to_string()]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/query-one", "string"])),
    };

    let params = if args.len() >= 3 {
        params_from_value(&args[2]).map_err(|e| e.message)?
    } else {
        vec![]
    };

    let opts = if args.len() == 4 {
        QueryOptions::from_value(&args[3]).map_err(|e| e.message)?
    } else {
        QueryOptions::default()
    };

    let connections = CONNECTIONS.lock();
    let conn = connections.get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let row = conn.query_one(sql, &params, &opts)
        .map_err(|e| e.message)?;

    Ok(row.map(row_to_value).unwrap_or(Value::Nil))
}

/// db/exec - SQL文を実行
pub fn native_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(MsgKey::DbNeed2To4Args, &["db/exec", &args.len().to_string()]));
    }

    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/exec", "string"])),
    };

    let params = if args.len() >= 3 {
        params_from_value(&args[2]).map_err(|e| e.message)?
    } else {
        vec![]
    };

    let opts = if args.len() == 4 {
        QueryOptions::from_value(&args[3]).map_err(|e| e.message)?
    } else {
        QueryOptions::default()
    };

    // 接続かトランザクションかを判別
    let affected = match extract_conn_or_tx(&args[0])? {
        ConnOrTx::Conn(conn_id) => {
            let connections = CONNECTIONS.lock();
            let conn = connections.get(&conn_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;
            conn.exec(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            let transactions = TRANSACTIONS.lock();
            let tx = transactions.get(&tx_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;
            tx.exec(sql, &params, &opts).map_err(|e| e.message)?
        }
    };

    Ok(Value::Integer(affected))
}

/// db/close - 接続を閉じる
pub fn native_close(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/close"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let mut connections = CONNECTIONS.lock();
    let conn = connections.remove(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    conn.close().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

/// db/sanitize - 値をサニタイズ
pub fn native_sanitize(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/sanitize"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let value = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/sanitize", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections.get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    Ok(Value::String(conn.sanitize(value)))
}

/// db/sanitize-identifier - 識別子をサニタイズ
pub fn native_sanitize_identifier(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/sanitize-identifier"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let name = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/sanitize-identifier", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections.get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    Ok(Value::String(conn.sanitize_identifier(name)))
}

/// db/escape-like - LIKE句のパターンをエスケープ
pub fn native_escape_like(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/escape-like"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/escape-like", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections.get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    Ok(Value::String(conn.escape_like(pattern)))
}

/// 接続IDを抽出
fn extract_conn_id(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) if s.starts_with("DbConnection:") => {
            Ok(s.strip_prefix("DbConnection:").unwrap().to_string())
        }
        _ => Err(fmt_msg(MsgKey::DbExpectedConnection, &[&format!("{:?}", value)])),
    }
}

/// 接続またはトランザクションを判別
enum ConnOrTx {
    Conn(String),
    Tx(String),
}

/// 接続IDまたはトランザクションIDを抽出
fn extract_conn_or_tx(value: &Value) -> Result<ConnOrTx, String> {
    match value {
        Value::String(s) if s.starts_with("DbConnection:") => {
            Ok(ConnOrTx::Conn(s.strip_prefix("DbConnection:").unwrap().to_string()))
        }
        Value::String(s) if s.starts_with("DbTransaction:") => {
            Ok(ConnOrTx::Tx(s.strip_prefix("DbTransaction:").unwrap().to_string()))
        }
        _ => Err(fmt_msg(MsgKey::DbExpectedConnectionOrTransaction, &[&format!("{:?}", value)])),
    }
}

/// トランザクションIDを生成
fn gen_tx_id() -> String {
    let mut id = NEXT_TX_ID.lock();
    let tx_id = format!("tx_{}", *id);
    *id += 1;
    tx_id
}

/// トランザクションIDを抽出
fn extract_tx_id(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) if s.starts_with("DbTransaction:") => {
            Ok(s.strip_prefix("DbTransaction:").unwrap().to_string())
        }
        _ => Err(fmt_msg(MsgKey::DbExpectedTransaction, &[&format!("{:?}", value)])),
    }
}

/// db/begin - トランザクションを開始
pub fn native_begin(args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["db/begin"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let opts = if args.len() == 2 {
        TransactionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        TransactionOptions::default()
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let tx = conn.begin(&opts).map_err(|e| e.message)?;

    // トランザクションを保存
    let tx_id = gen_tx_id();
    TRANSACTIONS.lock().insert(tx_id.clone(), tx);

    Ok(Value::String(format!("DbTransaction:{}", tx_id)))
}

/// db/commit - トランザクションをコミット
pub fn native_commit(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/commit"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    let mut transactions = TRANSACTIONS.lock();
    let tx = transactions
        .remove(&tx_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;

    tx.commit().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

/// db/rollback - トランザクションをロールバック
pub fn native_rollback(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/rollback"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    let mut transactions = TRANSACTIONS.lock();
    let tx = transactions
        .remove(&tx_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;

    tx.rollback().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_options() {
        let opts = Value::Map(HashMap::from([
            ("timeout".to_string(), Value::Integer(5000)),
            ("read-only".to_string(), Value::Bool(true)),
        ]));

        let conn_opts = ConnectionOptions::from_value(&opts).unwrap();
        assert_eq!(conn_opts.timeout_ms, Some(5000));
        assert!(conn_opts.read_only);
    }

    #[test]
    fn test_isolation_level() {
        assert_eq!(
            IsolationLevel::from_str("serializable"),
            Some(IsolationLevel::Serializable)
        );
        assert_eq!(
            IsolationLevel::from_str("read-committed"),
            Some(IsolationLevel::ReadCommitted)
        );
    }
}
