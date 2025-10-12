//! SQLiteドライバー実装
//!
//! SQLite 3を使用したデータベース接続を提供。
//! - 埋め込み型データベース
//! - 外部依存なし
//! - ACID準拠のトランザクション
//!
//! このモジュールは `db-sqlite` feature でコンパイルされます。

#![cfg(feature = "db-sqlite")]

use super::db::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use base64::Engine;
use parking_lot::Mutex;
use rusqlite::{params_from_iter, Connection as SqliteConn, Row as SqliteRow};
use std::sync::Arc;

/// SQLiteドライバー
pub struct SqliteDriver;

impl SqliteDriver {
    pub fn new() -> Self {
        Self
    }
}

impl DbDriver for SqliteDriver {
    fn connect(&self, url: &str, opts: &ConnectionOptions) -> DbResult<Arc<dyn DbConnection>> {
        // URL形式: "sqlite:path/to/db.db" または "sqlite::memory:"
        let path = url
            .strip_prefix("sqlite:")
            .ok_or_else(|| DbError::new("Invalid SQLite URL. Expected format: sqlite:path/to/db.db"))?;

        let conn = if opts.read_only {
            SqliteConn::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
        } else {
            SqliteConn::open(path)
        }
        .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToOpen, &[&e.to_string()])))?;

        // タイムアウト設定
        if let Some(timeout_ms) = opts.timeout_ms {
            conn.busy_timeout(std::time::Duration::from_millis(timeout_ms))
                .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToSetTimeout, &[&e.to_string()])))?;
        }

        Ok(Arc::new(SqliteConnection {
            conn: Arc::new(Mutex::new(conn)),
        }))
    }

    fn name(&self) -> &str {
        "sqlite"
    }
}

/// SQLite接続
pub struct SqliteConnection {
    conn: Arc<Mutex<SqliteConn>>,
}

impl SqliteConnection {
    /// Valueをrusqliteのパラメータに変換
    fn value_to_param(value: &Value) -> rusqlite::types::ToSqlOutput<'_> {
        use rusqlite::types::{ToSqlOutput, ValueRef};

        match value {
            Value::Nil => ToSqlOutput::Owned(rusqlite::types::Value::Null),
            Value::Bool(b) => ToSqlOutput::Owned(rusqlite::types::Value::Integer(if *b { 1 } else { 0 })),
            Value::Integer(i) => ToSqlOutput::Owned(rusqlite::types::Value::Integer(*i)),
            Value::Float(f) => ToSqlOutput::Owned(rusqlite::types::Value::Real(*f)),
            Value::String(s) => ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes())),
            // バイナリデータは今後の実装で対応
            _ => ToSqlOutput::Owned(rusqlite::types::Value::Text(value.to_string())),
        }
    }

    /// SQLite行をQi Valueに変換
    fn row_to_hashmap(row: &SqliteRow) -> DbResult<Row> {
        let mut map = Row::new();
        let column_count = row.as_ref().column_count();

        for i in 0..column_count {
            let column_name = row
                .as_ref()
                .column_name(i)
                .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToGetColumnName, &[&e.to_string()])))?
                .to_string();

            let value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => Value::Nil,
                Ok(rusqlite::types::ValueRef::Integer(i)) => Value::Integer(i),
                Ok(rusqlite::types::ValueRef::Real(f)) => Value::Float(f),
                Ok(rusqlite::types::ValueRef::Text(t)) => {
                    Value::String(String::from_utf8_lossy(t).to_string())
                }
                Ok(rusqlite::types::ValueRef::Blob(b)) => {
                    // バイナリデータは今後の実装で対応（今はbase64エンコードした文字列として返す）
                    Value::String(base64::engine::general_purpose::STANDARD.encode(b))
                }
                Err(e) => return Err(DbError::new(format!("Failed to get column value: {}", e))),
            };

            map.insert(column_name, value);
        }

        Ok(map)
    }
}

impl DbConnection for SqliteConnection {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToPrepare, &[&e.to_string()])))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(Self::value_to_param).collect();

        let rows = stmt
            .query(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToExecuteQuery, &[&e.to_string()])))?;

        let mut results = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            results.push(Self::row_to_hashmap(row)?);
        }

        Ok(results)
    }

    fn exec(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<i64> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToPrepare, &[&e.to_string()])))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(Self::value_to_param).collect();

        let affected = stmt
            .execute(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToExecuteStatement, &[&e.to_string()])))?;

        Ok(affected as i64)
    }

    fn begin(&self, opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>> {
        let conn = self.conn.lock();

        // トランザクション開始
        let isolation_sql = match opts.isolation {
            IsolationLevel::ReadUncommitted => "PRAGMA read_uncommitted = true; BEGIN;",
            IsolationLevel::ReadCommitted => "BEGIN;",
            IsolationLevel::RepeatableRead => "BEGIN;",
            IsolationLevel::Serializable => "BEGIN IMMEDIATE;",
        };

        conn.execute_batch(isolation_sql)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToBeginTransaction, &[&e.to_string()])))?;

        drop(conn); // ロックを解放

        Ok(Arc::new(SqliteTransaction {
            conn: self.conn.clone(),
            committed: Mutex::new(false),
        }))
    }

    fn close(&self) -> DbResult<()> {
        // SQLiteは自動的にDropで閉じられる
        Ok(())
    }

    fn sanitize(&self, value: &str) -> String {
        // SQLiteのルール: シングルクォートをダブル化
        value.replace('\'', "''")
    }

    fn sanitize_identifier(&self, name: &str) -> String {
        // SQLiteのルール: ダブルクォートで囲む
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn escape_like(&self, pattern: &str) -> String {
        // LIKE句の特殊文字 (%, _) をエスケープ
        pattern.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
    }

    fn driver_name(&self) -> &str {
        "sqlite"
    }

    // Phase 2: メタデータAPI
    fn tables(&self) -> DbResult<Vec<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .map_err(|e| DbError::new(format!("Failed to query tables: {}", e)))?;

        let rows = stmt
            .query([])
            .map_err(|e| DbError::new(format!("Failed to execute tables query: {}", e)))?;

        let mut tables = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            let name: String = row.get(0).map_err(|e| DbError::new(e.to_string()))?;
            tables.push(name);
        }

        Ok(tables)
    }

    fn columns(&self, table: &str) -> DbResult<Vec<ColumnInfo>> {
        let conn = self.conn.lock();
        let sql = format!("PRAGMA table_info({})", self.sanitize_identifier(table));
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::new(format!("Failed to query columns: {}", e)))?;

        let rows = stmt
            .query([])
            .map_err(|e| DbError::new(format!("Failed to execute columns query: {}", e)))?;

        let mut columns = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            let name: String = row.get(1).map_err(|e| DbError::new(e.to_string()))?;
            let data_type: String = row.get(2).map_err(|e| DbError::new(e.to_string()))?;
            let not_null: i32 = row.get(3).map_err(|e| DbError::new(e.to_string()))?;
            let default_value: Option<String> = row.get(4).ok();
            let primary_key: i32 = row.get(5).map_err(|e| DbError::new(e.to_string()))?;

            columns.push(ColumnInfo {
                name,
                data_type,
                nullable: not_null == 0,
                default_value,
                primary_key: primary_key > 0,
            });
        }

        Ok(columns)
    }

    fn indexes(&self, table: &str) -> DbResult<Vec<IndexInfo>> {
        let conn = self.conn.lock();
        let sql = format!("PRAGMA index_list({})", self.sanitize_identifier(table));
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::new(format!("Failed to query indexes: {}", e)))?;

        let rows = stmt
            .query([])
            .map_err(|e| DbError::new(format!("Failed to execute indexes query: {}", e)))?;

        let mut indexes = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            let name: String = row.get(1).map_err(|e| DbError::new(e.to_string()))?;
            let unique: i32 = row.get(2).map_err(|e| DbError::new(e.to_string()))?;

            // インデックスのカラムを取得
            let col_sql = format!("PRAGMA index_info({})", self.sanitize_identifier(&name));
            let mut col_stmt = conn
                .prepare(&col_sql)
                .map_err(|e| DbError::new(format!("Failed to query index columns: {}", e)))?;

            let col_rows = col_stmt
                .query([])
                .map_err(|e| DbError::new(format!("Failed to execute index columns query: {}", e)))?;

            let mut columns = Vec::new();
            let mut col_rows = col_rows;
            while let Some(col_row) = col_rows.next().map_err(|e| DbError::new(e.to_string()))? {
                let col_name: String = col_row.get(2).map_err(|e| DbError::new(e.to_string()))?;
                columns.push(col_name);
            }

            indexes.push(IndexInfo {
                name,
                table: table.to_string(),
                columns,
                unique: unique > 0,
            });
        }

        Ok(indexes)
    }

    fn foreign_keys(&self, table: &str) -> DbResult<Vec<ForeignKeyInfo>> {
        let conn = self.conn.lock();
        let sql = format!("PRAGMA foreign_key_list({})", self.sanitize_identifier(table));
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DbError::new(format!("Failed to query foreign keys: {}", e)))?;

        let rows = stmt
            .query([])
            .map_err(|e| DbError::new(format!("Failed to execute foreign keys query: {}", e)))?;

        let mut fkeys = Vec::new();
        let mut rows = rows;
        let mut current_id = -1;
        let mut current_fk: Option<ForeignKeyInfo> = None;

        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            let id: i32 = row.get(0).map_err(|e| DbError::new(e.to_string()))?;
            let referenced_table: String = row.get(2).map_err(|e| DbError::new(e.to_string()))?;
            let from_col: String = row.get(3).map_err(|e| DbError::new(e.to_string()))?;
            let to_col: String = row.get(4).map_err(|e| DbError::new(e.to_string()))?;

            if id != current_id {
                // 新しい外部キー
                if let Some(fk) = current_fk.take() {
                    fkeys.push(fk);
                }

                current_id = id;
                current_fk = Some(ForeignKeyInfo {
                    name: format!("fk_{}_{}", table, id),
                    table: table.to_string(),
                    columns: vec![from_col],
                    referenced_table,
                    referenced_columns: vec![to_col],
                });
            } else {
                // 既存の外部キーにカラムを追加
                if let Some(ref mut fk) = current_fk {
                    fk.columns.push(from_col);
                    fk.referenced_columns.push(to_col);
                }
            }
        }

        // 最後の外部キーを追加
        if let Some(fk) = current_fk {
            fkeys.push(fk);
        }

        Ok(fkeys)
    }

    fn call(&self, _name: &str, _params: &[Value]) -> DbResult<CallResult> {
        Err(DbError::new("SQLite does not support stored procedures/functions"))
    }

    fn supports(&self, feature: &str) -> bool {
        match feature {
            "transactions" => true,
            "prepared_statements" => true,
            "blob" => true,
            "foreign_keys" => true,
            "stored_procedures" => false,
            "stored_functions" => false,
            _ => false,
        }
    }

    fn driver_info(&self) -> DbResult<DriverInfo> {
        let conn = self.conn.lock();
        let version = rusqlite::version();
        let db_version = conn
            .query_row("SELECT sqlite_version()", [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| DbError::new(format!("Failed to get database version: {}", e)))?;

        Ok(DriverInfo {
            name: "sqlite".to_string(),
            version: version.to_string(),
            database_version: db_version,
        })
    }
}

/// SQLiteトランザクション
pub struct SqliteTransaction {
    conn: Arc<Mutex<SqliteConn>>,
    committed: Mutex<bool>,
}

impl DbTransaction for SqliteTransaction {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToPrepare, &[&e.to_string()])))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(SqliteConnection::value_to_param).collect();

        let rows = stmt
            .query(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToExecuteQuery, &[&e.to_string()])))?;

        let mut results = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows.next().map_err(|e| DbError::new(e.to_string()))? {
            results.push(SqliteConnection::row_to_hashmap(row)?);
        }

        Ok(results)
    }

    fn exec(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<i64> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToPrepare, &[&e.to_string()])))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(SqliteConnection::value_to_param).collect();

        let affected = stmt
            .execute(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToExecuteStatement, &[&e.to_string()])))?;

        Ok(affected as i64)
    }

    fn commit(self: Arc<Self>) -> DbResult<()> {
        let mut committed = self.committed.lock();
        if *committed {
            return Err(DbError::new("Transaction already committed or rolled back"));
        }

        let conn = self.conn.lock();
        conn.execute_batch("COMMIT;")
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToCommitTransaction, &[&e.to_string()])))?;

        *committed = true;
        Ok(())
    }

    fn rollback(self: Arc<Self>) -> DbResult<()> {
        let mut committed = self.committed.lock();
        if *committed {
            return Err(DbError::new("Transaction already committed or rolled back"));
        }

        let conn = self.conn.lock();
        conn.execute_batch("ROLLBACK;")
            .map_err(|e| DbError::new(fmt_msg(MsgKey::SqliteFailedToRollbackTransaction, &[&e.to_string()])))?;

        *committed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_connection() {
        let driver = SqliteDriver::new();
        let opts = ConnectionOptions::default();
        let conn = driver.connect("sqlite::memory:", &opts).unwrap();

        // テーブル作成
        let query_opts = QueryOptions::default();
        conn.exec(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
            &[],
            &query_opts,
        )
        .unwrap();

        // データ挿入
        conn.exec(
            "INSERT INTO test (name) VALUES (?)",
            &[Value::String("Alice".to_string())],
            &query_opts,
        )
        .unwrap();

        // クエリ実行
        let rows = conn
            .query("SELECT * FROM test", &[], &query_opts)
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_sanitize() {
        let driver = SqliteDriver::new();
        let opts = ConnectionOptions::default();
        let conn = driver.connect("sqlite::memory:", &opts).unwrap();

        assert_eq!(conn.sanitize("O'Reilly"), "O''Reilly");
        assert_eq!(conn.sanitize_identifier("table\"name"), "\"table\"\"name\"");
        assert_eq!(conn.escape_like("50%_off"), "50\\%\\_off");
    }
}
