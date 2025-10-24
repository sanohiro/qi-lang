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

/// カラム情報
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

/// インデックス情報
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// 外部キー情報
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
}

/// ストアドプロシージャ/ファンクション呼び出しの結果
#[derive(Debug, Clone)]
pub enum CallResult {
    /// 関数の戻り値
    Value(Value),
    /// プロシージャの結果セット
    Rows(Rows),
    /// 複数の結果セット
    Multiple(Vec<Rows>),
}

/// ドライバー情報
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
    pub database_version: String,
}

/// クエリ情報
#[derive(Debug, Clone)]
pub struct QueryInfo {
    pub columns: Vec<ColumnInfo>,
    pub parameter_count: usize,
}

/// トランザクション分離レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl std::str::FromStr for IsolationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read-uncommitted" => Ok(Self::ReadUncommitted),
            "read-committed" => Ok(Self::ReadCommitted),
            "repeatable-read" => Ok(Self::RepeatableRead),
            "serializable" => Ok(Self::Serializable),
            _ => Err(fmt_msg(MsgKey::InvalidIsolationLevel, &[s])),
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
                options.isolation = iso.parse().map_err(DbError::new)?;
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

    // Phase 2: メタデータAPI
    /// テーブル一覧を取得
    fn tables(&self) -> DbResult<Vec<String>>;

    /// テーブルのカラム情報を取得
    fn columns(&self, table: &str) -> DbResult<Vec<ColumnInfo>>;

    /// テーブルのインデックス一覧を取得
    fn indexes(&self, table: &str) -> DbResult<Vec<IndexInfo>>;

    /// テーブルの外部キー一覧を取得
    fn foreign_keys(&self, table: &str) -> DbResult<Vec<ForeignKeyInfo>>;

    /// ストアドプロシージャ/ファンクションを呼び出す
    fn call(&self, name: &str, params: &[Value]) -> DbResult<CallResult>;

    /// 機能をサポートしているか確認
    fn supports(&self, feature: &str) -> bool;

    /// ドライバー情報を取得
    fn driver_info(&self) -> DbResult<DriverInfo>;

    /// クエリのメタデータを取得（実行せずにカラム情報を取得）
    fn query_info(&self, sql: &str) -> DbResult<QueryInfo>;
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
    Value::Map(row.into())
}

/// 複数行をQi ValueのVectorに変換
pub fn rows_to_value(rows: Rows) -> Value {
    Value::Vector(rows.into_iter().map(row_to_value).collect())
}

/// Qi Valueからパラメータを抽出
pub fn params_from_value(params: &Value) -> DbResult<Vec<Value>> {
    match params {
        Value::Vector(vec) => Ok(vec.iter().cloned().collect()),
        Value::Nil => Ok(vec![]),
        _ => Err(DbError::new("Parameters must be a vector")),
    }
}

// ========================================
// Qi Builtin関数
// ========================================

#[cfg(feature = "db-sqlite")]
use super::sqlite::SqliteDriver;
use crate::i18n::fmt_msg;
use crate::i18n::MsgKey;
use parking_lot::Mutex;

/// コネクションプール
#[derive(Clone)]
pub struct DbPool {
    url: String,
    opts: ConnectionOptions,
    max_connections: usize,
    available: Arc<Mutex<Vec<Arc<dyn DbConnection>>>>,
    in_use_count: Arc<Mutex<usize>>,
}

impl DbPool {
    /// 新しいコネクションプールを作成
    pub fn new(url: String, opts: ConnectionOptions, max_connections: usize) -> Self {
        Self {
            url,
            opts,
            max_connections,
            available: Arc::new(Mutex::new(Vec::new())),
            in_use_count: Arc::new(Mutex::new(0)),
        }
    }

    /// プールから接続を取得（利用可能な接続がなければ新規作成）
    pub fn acquire(&self) -> DbResult<Arc<dyn DbConnection>> {
        // まず利用可能な接続があるか確認
        let mut available = self.available.lock();
        if let Some(conn) = available.pop() {
            let mut in_use = self.in_use_count.lock();
            *in_use += 1;
            return Ok(conn);
        }

        // 利用可能な接続がない場合、新規作成を試みる
        drop(available); // ロックを解放
        let in_use = self.in_use_count.lock();
        let total = *in_use + self.available.lock().len();

        if total >= self.max_connections {
            return Err(DbError::new(format!(
                "Connection pool exhausted (max: {})",
                self.max_connections
            )));
        }
        drop(in_use); // ロックを解放

        // 新しい接続を作成
        let driver: Box<dyn DbDriver> = if self.url.starts_with("sqlite:") {
            #[cfg(feature = "db-sqlite")]
            {
                Box::new(SqliteDriver::new())
            }
            #[cfg(not(feature = "db-sqlite"))]
            {
                return Err(DbError::new("SQLite driver not enabled"));
            }
        } else {
            return Err(DbError::new(format!("Unsupported URL: {}", self.url)));
        };

        let conn = driver.connect(&self.url, &self.opts)?;
        let mut in_use = self.in_use_count.lock();
        *in_use += 1;
        Ok(conn)
    }

    /// 接続をプールに返却
    pub fn release(&self, conn: Arc<dyn DbConnection>) {
        let mut available = self.available.lock();
        available.push(conn);
        let mut in_use = self.in_use_count.lock();
        *in_use = in_use.saturating_sub(1);
    }

    /// プール全体をクローズ
    pub fn close(&self) -> DbResult<()> {
        let mut available = self.available.lock();
        for conn in available.drain(..) {
            conn.close()?;
        }
        let mut in_use = self.in_use_count.lock();
        *in_use = 0;
        Ok(())
    }

    /// プールの統計情報を取得
    pub fn stats(&self) -> (usize, usize, usize) {
        let available = self.available.lock().len();
        let in_use = *self.in_use_count.lock();
        (available, in_use, self.max_connections)
    }
}

lazy_static::lazy_static! {
    /// グローバル接続マネージャー
    static ref CONNECTIONS: Mutex<HashMap<String, Arc<dyn DbConnection>>> = Mutex::new(HashMap::new());
    static ref NEXT_CONN_ID: Mutex<usize> = Mutex::new(0);

    /// グローバルトランザクションマネージャー
    static ref TRANSACTIONS: Mutex<HashMap<String, Arc<dyn DbTransaction>>> = Mutex::new(HashMap::new());
    static ref NEXT_TX_ID: Mutex<usize> = Mutex::new(0);

    /// グローバルプールマネージャー
    static ref POOLS: Mutex<HashMap<String, DbPool>> = Mutex::new(HashMap::new());
    static ref NEXT_POOL_ID: Mutex<usize> = Mutex::new(0);
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
        ConnectionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        ConnectionOptions::default()
    };

    // URLからドライバーを判定
    let driver: Box<dyn DbDriver> = if url.starts_with("sqlite:") {
        #[cfg(feature = "db-sqlite")]
        {
            Box::new(SqliteDriver::new())
        }
        #[cfg(not(feature = "db-sqlite"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["sqlite (feature not enabled)"],
            ));
        }
    } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        #[cfg(feature = "db-postgres")]
        {
            Box::new(super::postgres::PostgresDriver::new())
        }
        #[cfg(not(feature = "db-postgres"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["postgres (feature not enabled)"],
            ));
        }
    } else if url.starts_with("mysql://") {
        #[cfg(feature = "db-mysql")]
        {
            Box::new(super::mysql::MysqlDriver::new())
        }
        #[cfg(not(feature = "db-mysql"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["mysql (feature not enabled)"],
            ));
        }
    } else {
        return Err(fmt_msg(MsgKey::DbUnsupportedUrl, &[url]));
    };

    let conn = driver.connect(url, &opts).map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/query - SQLクエリを実行
pub fn native_query(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(
            MsgKey::DbNeed2To4Args,
            &["db/query", &args.len().to_string()],
        ));
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
            let conn = connections
                .get(&conn_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;
            conn.query(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            let transactions = TRANSACTIONS.lock();
            let tx = transactions
                .get(&tx_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;
            tx.query(sql, &params, &opts).map_err(|e| e.message)?
        }
    };

    Ok(rows_to_value(rows))
}

/// db/query-one - SQLクエリを実行して最初の1行のみ取得
pub fn native_query_one(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(
            MsgKey::DbNeed2To4Args,
            &["db/query-one", &args.len().to_string()],
        ));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/query-one", "string"],
            ))
        }
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
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let row = conn.query_one(sql, &params, &opts).map_err(|e| e.message)?;

    Ok(row.map(row_to_value).unwrap_or(Value::Nil))
}

/// db/exec - SQL文を実行
pub fn native_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(
            MsgKey::DbNeed2To4Args,
            &["db/exec", &args.len().to_string()],
        ));
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
            let conn = connections
                .get(&conn_id)
                .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;
            conn.exec(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            let transactions = TRANSACTIONS.lock();
            let tx = transactions
                .get(&tx_id)
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
    let conn = connections
        .remove(&conn_id)
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
    let conn = connections
        .get(&conn_id)
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
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/sanitize-identifier", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
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
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/escape-like", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    Ok(Value::String(conn.escape_like(pattern)))
}

/// 接続IDを抽出
fn extract_conn_id(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) if s.starts_with("DbConnection:") => {
            Ok(s.strip_prefix("DbConnection:").unwrap().to_string())
        }
        _ => Err(fmt_msg(
            MsgKey::DbExpectedConnection,
            &[&format!("{:?}", value)],
        )),
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
        Value::String(s) if s.starts_with("DbConnection:") => Ok(ConnOrTx::Conn(
            s.strip_prefix("DbConnection:").unwrap().to_string(),
        )),
        Value::String(s) if s.starts_with("DbTransaction:") => Ok(ConnOrTx::Tx(
            s.strip_prefix("DbTransaction:").unwrap().to_string(),
        )),
        _ => Err(fmt_msg(
            MsgKey::DbExpectedConnectionOrTransaction,
            &[&format!("{:?}", value)],
        )),
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
        _ => Err(fmt_msg(
            MsgKey::DbExpectedTransaction,
            &[&format!("{:?}", value)],
        )),
    }
}

/// db/begin - トランザクションを開始
pub fn native_begin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
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

/// db/tables - テーブル一覧を取得
pub fn native_tables(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/tables"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let tables = conn.tables().map_err(|e| e.message)?;

    Ok(Value::Vector(
        tables.into_iter().map(Value::String).collect(),
    ))
}

/// db/columns - テーブルのカラム情報を取得
pub fn native_columns(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/columns"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/columns", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let columns = conn.columns(table).map_err(|e| e.message)?;

    // Vec<ColumnInfo>をValue::Vectorに変換
    let result = columns
        .into_iter()
        .map(|col| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(col.name));
            map.insert("type".to_string(), Value::String(col.data_type));
            map.insert("nullable".to_string(), Value::Bool(col.nullable));
            map.insert(
                "default".to_string(),
                col.default_value.map(Value::String).unwrap_or(Value::Nil),
            );
            map.insert("primary_key".to_string(), Value::Bool(col.primary_key));
            Value::Map(map.into())
        })
        .collect();

    Ok(Value::Vector(result))
}

/// db/indexes - テーブルのインデックス一覧を取得
pub fn native_indexes(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/indexes"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/indexes", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let indexes = conn.indexes(table).map_err(|e| e.message)?;

    // Vec<IndexInfo>をValue::Vectorに変換
    let result = indexes
        .into_iter()
        .map(|idx| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(idx.name));
            map.insert("table".to_string(), Value::String(idx.table));
            map.insert(
                "columns".to_string(),
                Value::Vector(idx.columns.into_iter().map(Value::String).collect()),
            );
            map.insert("unique".to_string(), Value::Bool(idx.unique));
            Value::Map(map.into())
        })
        .collect();

    Ok(Value::Vector(result))
}

/// db/foreign-keys - テーブルの外部キー一覧を取得
pub fn native_foreign_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/foreign-keys"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/foreign-keys", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let foreign_keys = conn.foreign_keys(table).map_err(|e| e.message)?;

    // Vec<ForeignKeyInfo>をValue::Vectorに変換
    let result = foreign_keys
        .into_iter()
        .map(|fk| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(fk.name));
            map.insert("table".to_string(), Value::String(fk.table));
            map.insert(
                "columns".to_string(),
                Value::Vector(fk.columns.into_iter().map(Value::String).collect()),
            );
            map.insert(
                "referenced_table".to_string(),
                Value::String(fk.referenced_table),
            );
            map.insert(
                "referenced_columns".to_string(),
                Value::Vector(
                    fk.referenced_columns
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
            Value::Map(map.into())
        })
        .collect();

    Ok(Value::Vector(result))
}

/// db/call - ストアドプロシージャ/ファンクションを呼び出す
pub fn native_call(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["db/call"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let name = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/call", "string"])),
    };

    let params = if args.len() == 3 {
        params_from_value(&args[2]).map_err(|e| e.message)?
    } else {
        vec![]
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let call_result = conn.call(name, &params).map_err(|e| e.message)?;

    // CallResultをValueに変換
    let result = match call_result {
        CallResult::Value(v) => v,
        CallResult::Rows(rows) => rows_to_value(rows),
        CallResult::Multiple(multiple) => {
            Value::Vector(multiple.into_iter().map(rows_to_value).collect())
        }
    };

    Ok(result)
}

/// db/supports? - 機能をサポートしているか確認
pub fn native_supports(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/supports?"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let feature = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/supports?", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let supported = conn.supports(feature);

    Ok(Value::Bool(supported))
}

/// db/driver-info - ドライバー情報を取得
pub fn native_driver_info(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/driver-info"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let info = conn.driver_info().map_err(|e| e.message)?;

    // DriverInfoをマップに変換
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String(info.name));
    map.insert("version".to_string(), Value::String(info.version));
    map.insert(
        "database_version".to_string(),
        Value::String(info.database_version),
    );

    Ok(Value::Map(map.into()))
}

/// db/query-info - クエリのメタデータを取得
pub fn native_query_info(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/query-info"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/query-info", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let info = conn.query_info(sql).map_err(|e| e.message)?;

    // QueryInfoをマップに変換
    let columns = info
        .columns
        .into_iter()
        .map(|col| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(col.name));
            map.insert("type".to_string(), Value::String(col.data_type));
            map.insert("nullable".to_string(), Value::Bool(col.nullable));
            map.insert(
                "default".to_string(),
                col.default_value.map(Value::String).unwrap_or(Value::Nil),
            );
            map.insert("primary_key".to_string(), Value::Bool(col.primary_key));
            Value::Map(map.into())
        })
        .collect();

    let mut result = HashMap::new();
    result.insert("columns".to_string(), Value::Vector(columns));
    result.insert(
        "parameter_count".to_string(),
        Value::Integer(info.parameter_count as i64),
    );

    Ok(Value::Map(result.into()))
}

// ========================================
// Phase 3: コネクションプーリング関数
// ========================================

/// プールIDを生成
fn gen_pool_id() -> String {
    let mut id = NEXT_POOL_ID.lock();
    let pool_id = format!("pool_{}", *id);
    *id += 1;
    pool_id
}

/// プールIDを抽出
fn extract_pool_id(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) if s.starts_with("DbPool:") => {
            Ok(s.strip_prefix("DbPool:").unwrap().to_string())
        }
        _ => Err(fmt_msg(MsgKey::DbExpectedPool, &[&format!("{:?}", value)])),
    }
}

/// db/create-pool - コネクションプールを作成
pub fn native_create_pool(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 3 {
        return Err(fmt_msg(
            MsgKey::DbNeed1To3Args,
            &["db/create-pool", &args.len().to_string()],
        ));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["db/create-pool", "string"],
            ))
        }
    };

    let opts = if args.len() >= 2 {
        ConnectionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        ConnectionOptions::default()
    };

    let max_connections = if args.len() >= 3 {
        match &args[2] {
            Value::Integer(n) if *n > 0 => *n as usize,
            Value::Integer(_) => {
                return Err(fmt_msg(
                    MsgKey::DbInvalidPoolSize,
                    &["db/create-pool", "positive integer"],
                ))
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::ThirdArgMustBe,
                    &["db/create-pool", "positive integer"],
                ))
            }
        }
    } else {
        10 // デフォルトは10接続
    };

    // プールを作成
    let pool = DbPool::new(url, opts, max_connections);
    let pool_id = gen_pool_id();
    POOLS.lock().insert(pool_id.clone(), pool);

    Ok(Value::String(format!("DbPool:{}", pool_id)))
}

/// db/pool-acquire - プールから接続を取得
pub fn native_pool_acquire(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-acquire"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let conn = pool.acquire().map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/pool-release - 接続をプールに返却
pub fn native_pool_release(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/pool-release"]));
    }

    let pool_id = extract_pool_id(&args[0])?;
    let conn_id = extract_conn_id(&args[1])?;

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let mut connections = CONNECTIONS.lock();
    let conn = connections
        .remove(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    pool.release(conn);

    Ok(Value::Nil)
}

/// db/pool-close - プール全体をクローズ
pub fn native_pool_close(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-close"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    let mut pools = POOLS.lock();
    let pool = pools
        .remove(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    pool.close().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

/// db/pool-stats - プールの統計情報を取得
pub fn native_pool_stats(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-stats"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let (available, in_use, max) = pool.stats();

    let mut map = HashMap::new();
    map.insert("available".to_string(), Value::Integer(available as i64));
    map.insert("in_use".to_string(), Value::Integer(in_use as i64));
    map.insert("max".to_string(), Value::Integer(max as i64));

    Ok(Value::Map(map.into()))
}

/// データベース関数配列（24個）
/// @qi-doc:category db
/// @qi-doc:functions connect, close, exec, query, query-one, prepare, exec-prepared, query-prepared, begin, commit, rollback, escape-string, escape-identifier, table-list, column-list, table-exists?, column-exists?, create-table, drop-table, add-column, drop-column, list-indexes, create-index, drop-index
pub const FUNCTIONS: super::NativeFunctions = &[
    // Phase 1: 基本操作、サニタイズ（8個）
    ("db/connect", native_connect),
    ("db/query", native_query),
    ("db/query-one", native_query_one),
    ("db/exec", native_exec),
    ("db/close", native_close),
    ("db/sanitize", native_sanitize),
    ("db/sanitize-identifier", native_sanitize_identifier),
    ("db/escape-like", native_escape_like),
    // Phase 2: トランザクション、メタデータAPI（11個）
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
    // Phase 3: コネクションプーリング（5個）
    ("db/create-pool", native_create_pool),
    ("db/pool-acquire", native_pool_acquire),
    ("db/pool-release", native_pool_release),
    ("db/pool-close", native_pool_close),
    ("db/pool-stats", native_pool_stats),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_options() {
        let opts = Value::Map(
            HashMap::from([
                ("timeout".to_string(), Value::Integer(5000)),
                ("read-only".to_string(), Value::Bool(true)),
            ])
            .into(),
        );

        let conn_opts = ConnectionOptions::from_value(&opts).unwrap();
        assert_eq!(conn_opts.timeout_ms, Some(5000));
        assert!(conn_opts.read_only);
    }

    #[test]
    fn test_isolation_level() {
        assert_eq!(
            "serializable".parse::<IsolationLevel>(),
            Ok(IsolationLevel::Serializable)
        );
        assert_eq!(
            "read-committed".parse::<IsolationLevel>(),
            Ok(IsolationLevel::ReadCommitted)
        );
    }
}
