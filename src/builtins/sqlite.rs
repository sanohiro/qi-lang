//! SQLiteドライバー実装
//!
//! SQLite 3を使用したデータベース接続を提供。
//! - 埋め込み型データベース
//! - 外部依存なし
//! - ACID準拠のトランザクション

use super::db::*;
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
        .map_err(|e| DbError::new(format!("Failed to open SQLite database: {}", e)))?;

        // タイムアウト設定
        if let Some(timeout_ms) = opts.timeout_ms {
            conn.busy_timeout(std::time::Duration::from_millis(timeout_ms))
                .map_err(|e| DbError::new(format!("Failed to set timeout: {}", e)))?;
        }

        Ok(Arc::new(SqliteConnection {
            conn: Mutex::new(conn),
        }))
    }

    fn name(&self) -> &str {
        "sqlite"
    }
}

/// SQLite接続
pub struct SqliteConnection {
    conn: Mutex<SqliteConn>,
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
                .map_err(|e| DbError::new(format!("Failed to get column name: {}", e)))?
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
            .map_err(|e| DbError::new(format!("Failed to prepare statement: {}", e)))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(Self::value_to_param).collect();

        let rows = stmt
            .query(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(format!("Failed to execute query: {}", e)))?;

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
            .map_err(|e| DbError::new(format!("Failed to prepare statement: {}", e)))?;

        // パラメータをrusqliteの形式に変換
        let param_refs: Vec<_> = params.iter().map(Self::value_to_param).collect();

        let affected = stmt
            .execute(params_from_iter(param_refs.iter()))
            .map_err(|e| DbError::new(format!("Failed to execute statement: {}", e)))?;

        Ok(affected as i64)
    }

    fn begin(&self, _opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>> {
        // TODO: Phase 2で実装
        Err(DbError::new("Transactions are not yet implemented (coming in Phase 2)"))
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
}

// TODO: Phase 2でトランザクション機能を実装

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
