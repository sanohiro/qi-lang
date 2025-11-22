use super::*;
use crate::builtins::db::traits::*;
use crate::builtins::db::types::*;
use parking_lot::Mutex;

#[cfg(feature = "db-sqlite")]
use crate::builtins::sqlite::SqliteDriver;

#[cfg(feature = "db-postgres")]
use crate::builtins::postgres::PostgresDriver;

#[cfg(feature = "db-mysql")]
use crate::builtins::mysql::MysqlDriver;

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
        // 競合状態を防ぐため、両方のカウントを1つのクリティカルセクションで取得
        let available_count = available.len();
        drop(available); // ロックを解放

        let mut in_use = self.in_use_count.lock();
        let total = *in_use + available_count;

        if total >= self.max_connections {
            return Err(DbError::new(format!(
                "Connection pool exhausted (max: {})",
                self.max_connections
            )));
        }

        // in_useを先にインクリメント（競合状態を防ぐ）
        // これにより、他のスレッドが同時にmax_connectionsを超えることを防ぐ
        *in_use += 1;
        drop(in_use);

        // 新しい接続を作成（失敗したらin_useをデクリメント）
        let driver: Box<dyn DbDriver> = if self.url.starts_with("sqlite:") {
            #[cfg(feature = "db-sqlite")]
            {
                Box::new(SqliteDriver::new())
            }
            #[cfg(not(feature = "db-sqlite"))]
            {
                return Err(DbError::new("SQLite driver not enabled"));
            }
        } else if self.url.starts_with("postgres://") || self.url.starts_with("postgresql://") {
            #[cfg(feature = "db-postgres")]
            {
                Box::new(PostgresDriver::new())
            }
            #[cfg(not(feature = "db-postgres"))]
            {
                return Err(DbError::new("PostgreSQL driver not enabled"));
            }
        } else if self.url.starts_with("mysql://") {
            #[cfg(feature = "db-mysql")]
            {
                Box::new(MysqlDriver::new())
            }
            #[cfg(not(feature = "db-mysql"))]
            {
                return Err(DbError::new("MySQL driver not enabled"));
            }
        } else {
            // サポートされていないURL
            let mut in_use = self.in_use_count.lock();
            *in_use = in_use.saturating_sub(1);
            return Err(DbError::new(format!("Unsupported URL: {}", self.url)));
        };

        // 接続を作成（失敗した場合はin_useをデクリメント）
        match driver.connect(&self.url, &self.opts) {
            Ok(conn) => Ok(conn),
            Err(e) => {
                let mut in_use = self.in_use_count.lock();
                *in_use = in_use.saturating_sub(1);
                Err(e)
            }
        }
    }

    /// 接続をプールに返却
    pub fn release(&self, conn: Arc<dyn DbConnection>) {
        // 不変条件を守るため、先にin_useをデクリメントしてからavailableに追加
        let mut in_use = self.in_use_count.lock();
        *in_use = in_use.saturating_sub(1);
        drop(in_use);

        let mut available = self.available.lock();
        available.push(conn);
    }

    /// プール全体をクローズ
    pub fn close(&self) -> DbResult<()> {
        let mut available = self.available.lock();

        // すべての接続をクローズ（エラーを収集）
        let mut errors = Vec::new();
        for conn in available.drain(..) {
            if let Err(e) = conn.close() {
                errors.push(e.message);
            }
        }

        // in_useを0にリセット（エラーがあってもリセット）
        let mut in_use = self.in_use_count.lock();
        *in_use = 0;

        // エラーがあった場合は報告
        if !errors.is_empty() {
            return Err(DbError::new(format!(
                "Failed to close {} connection(s): {}",
                errors.len(),
                errors.join(", ")
            )));
        }

        Ok(())
    }

    /// プールの統計情報を取得
    pub fn stats(&self) -> (usize, usize, usize) {
        // 一貫したスナップショットを返すため、両方のロックを同時に保持
        let available = self.available.lock();
        let in_use = self.in_use_count.lock();
        (available.len(), *in_use, self.max_connections)
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
    pub(super) static ref POOLS: Mutex<HashMap<String, DbPool>> = Mutex::new(HashMap::new());
    static ref NEXT_POOL_ID: Mutex<usize> = Mutex::new(0);

    /// プール接続の追跡（conn_id -> pool_id）
    /// プールから取得した接続を追跡し、db/closeではなくdb/pool-releaseを使うよう強制する
    pub(super) static ref POOLED_CONNECTIONS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());

    /// トランザクション→プールマッピング（tx_id -> (pool_id, conn_id)）
    /// トランザクション中の接続がdb/pool-releaseで誤って返却されないようにするために使用
    pub(super) static ref TRANSACTION_POOLS: Mutex<HashMap<String, (String, String)>> = Mutex::new(HashMap::new());
}
