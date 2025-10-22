//! MySQLドライバー実装
//!
//! MySQLデータベース接続を提供。
//! - リモート接続対応
//! - ACID準拠のトランザクション
//! - プリペアドステートメント
//!
//! このモジュールは `db-mysql` feature でコンパイルされます。

#![cfg(feature = "db-mysql")]

use super::db::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use mysql_async::prelude::*;
use mysql_async::{Conn as MyConn, Opts, Row as MyRow};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// MySQLドライバー
pub struct MysqlDriver;

impl Default for MysqlDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl MysqlDriver {
    pub fn new() -> Self {
        Self
    }
}

impl DbDriver for MysqlDriver {
    fn connect(&self, url: &str, _opts: &ConnectionOptions) -> DbResult<Arc<dyn DbConnection>> {
        // URL形式: "mysql://user:pass@host:port/dbname"
        let conn_str = if url.starts_with("mysql://") {
            url
        } else {
            return Err(DbError::new(
                "Invalid MySQL URL. Expected format: mysql://user:pass@host:port/dbname",
            ));
        };

        // 接続オプションを解析
        let opts = Opts::from_url(conn_str)
            .map_err(|e| DbError::new(fmt_msg(MsgKey::DbFailedToConnect, &[&e.to_string()])))?;

        // tokioランタイムを作成
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DbError::new(format!("Failed to create runtime: {}", e)))?;

        // 接続を確立
        let conn = rt
            .block_on(async { MyConn::new(opts).await })
            .map_err(|e| DbError::new(fmt_msg(MsgKey::DbFailedToConnect, &[&e.to_string()])))?;

        Ok(Arc::new(MysqlConnection {
            conn: Arc::new(Mutex::new(conn)),
            runtime: Arc::new(Mutex::new(rt)),
        }))
    }

    fn name(&self) -> &str {
        "mysql"
    }
}

/// MySQL接続
pub struct MysqlConnection {
    conn: Arc<Mutex<MyConn>>,
    runtime: Arc<Mutex<tokio::runtime::Runtime>>,
}

impl MysqlConnection {
    /// Valueをmysql_asyncのパラメータに変換
    fn value_to_mysql_value(value: &Value) -> mysql_async::Value {
        match value {
            Value::Nil => mysql_async::Value::NULL,
            Value::Bool(b) => mysql_async::Value::Int(if *b { 1 } else { 0 }),
            Value::Integer(i) => mysql_async::Value::Int(*i),
            Value::Float(f) => mysql_async::Value::Double(*f),
            Value::String(s) => mysql_async::Value::Bytes(s.as_bytes().to_vec()),
            _ => mysql_async::Value::Bytes(value.to_string().as_bytes().to_vec()),
        }
    }

    /// MySQL行をQi Valueに変換
    fn row_to_hashmap(row: &MyRow) -> DbResult<Row> {
        let mut map = Row::new();
        let columns = row.columns_ref();

        for (idx, column) in columns.iter().enumerate() {
            let column_name = column.name_str().to_string();

            // まず文字列として取得を試す（最も一般的）
            let value = if let Some(Ok(v)) = row.get_opt::<String, _>(idx) {
                // 文字列として取得できた場合、数値への変換を試みる
                if let Ok(i) = v.parse::<i64>() {
                    Value::Integer(i)
                } else if let Ok(f) = v.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::String(v)
                }
            } else if let Some(Ok(v)) = row.get_opt::<i64, _>(idx) {
                Value::Integer(v)
            } else if let Some(Ok(v)) = row.get_opt::<i32, _>(idx) {
                Value::Integer(v as i64)
            } else if let Some(Ok(v)) = row.get_opt::<f64, _>(idx) {
                Value::Float(v)
            } else if let Some(Ok(v)) = row.get_opt::<bool, _>(idx) {
                Value::Bool(v)
            } else {
                Value::Nil
            };

            map.insert(column_name, value);
        }

        Ok(map)
    }
}

impl DbConnection for MysqlConnection {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        // パラメータを変換
        let mysql_params: Vec<mysql_async::Value> =
            params.iter().map(Self::value_to_mysql_value).collect();

        let rows: Vec<MyRow> = runtime
            .block_on(async {
                if mysql_params.is_empty() {
                    conn.query(sql).await
                } else {
                    conn.exec(sql, mysql_params).await
                }
            })
            .map_err(|e| {
                DbError::new(fmt_msg(MsgKey::DbFailedToExecuteQuery, &[&e.to_string()]))
            })?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(Self::row_to_hashmap(row)?);
        }

        Ok(results)
    }

    fn exec(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<i64> {
        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        // パラメータを変換
        let mysql_params: Vec<mysql_async::Value> =
            params.iter().map(Self::value_to_mysql_value).collect();

        let affected = runtime
            .block_on(async {
                if mysql_params.is_empty() {
                    conn.query_drop(sql).await?;
                } else {
                    conn.exec_drop(sql, mysql_params).await?;
                }
                Ok::<_, mysql_async::Error>(conn.affected_rows())
            })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToExecuteStatement,
                    &[&e.to_string()],
                ))
            })?;

        Ok(affected as i64)
    }

    fn begin(&self, _opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>> {
        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        // トランザクション開始
        runtime
            .block_on(async { conn.query_drop("START TRANSACTION").await })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToBeginTransaction,
                    &[&e.to_string()],
                ))
            })?;

        Ok(Arc::new(MysqlTransaction {
            conn: self.conn.clone(),
            runtime: self.runtime.clone(),
            committed: Mutex::new(false),
        }))
    }

    fn close(&self) -> DbResult<()> {
        // MySQLコネクションは明示的にdisconnectする必要はない
        // Dropトレイトで自動的にクリーンアップされる
        Ok(())
    }

    fn sanitize(&self, value: &str) -> String {
        // シングルクォートをエスケープ
        value.replace('\'', "''")
    }

    fn sanitize_identifier(&self, name: &str) -> String {
        // バッククォートで囲む（MySQL識別子）
        format!("`{}`", name.replace('`', "``"))
    }

    fn escape_like(&self, pattern: &str) -> String {
        // LIKE句のメタキャラクタをエスケープ
        pattern
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }

    fn driver_name(&self) -> &str {
        "mysql"
    }

    fn tables(&self) -> DbResult<Vec<String>> {
        let sql = "SHOW TABLES";
        let rows = self.query(sql, &[], &QueryOptions::default())?;

        let tables = rows
            .into_iter()
            .filter_map(|row| {
                row.values().next().and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(tables)
    }

    fn columns(&self, table: &str) -> DbResult<Vec<ColumnInfo>> {
        let sql = format!("DESCRIBE `{}`", table.replace('`', "``"));
        let rows = self.query(&sql, &[], &QueryOptions::default())?;

        let columns = rows
            .into_iter()
            .filter_map(|row| {
                let name = row.get("Field")?.as_string()?;
                let data_type = row.get("Type")?.as_string()?;
                let nullable = row.get("Null")?.as_string()? == "YES";
                let default_value = row.get("Default").and_then(|v| v.as_string());
                let primary_key = row.get("Key")?.as_string()? == "PRI";

                Some(ColumnInfo {
                    name,
                    data_type,
                    nullable,
                    default_value,
                    primary_key,
                })
            })
            .collect();

        Ok(columns)
    }

    fn indexes(&self, table: &str) -> DbResult<Vec<IndexInfo>> {
        let sql = format!("SHOW INDEX FROM `{}`", table.replace('`', "``"));
        let rows = self.query(&sql, &[], &QueryOptions::default())?;

        let mut indexes_map: HashMap<String, IndexInfo> = HashMap::new();

        for row in rows {
            if let (Some(name), Some(column)) = (
                row.get("Key_name").and_then(|v| v.as_string()),
                row.get("Column_name").and_then(|v| v.as_string()),
            ) {
                let unique = row
                    .get("Non_unique")
                    .and_then(|v| {
                        if let Value::Integer(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(1)
                    == 0;

                indexes_map
                    .entry(name.clone())
                    .or_insert_with(|| IndexInfo {
                        name: name.clone(),
                        table: table.to_string(),
                        columns: vec![],
                        unique,
                    })
                    .columns
                    .push(column);
            }
        }

        Ok(indexes_map.into_values().collect())
    }

    fn foreign_keys(&self, _table: &str) -> DbResult<Vec<ForeignKeyInfo>> {
        // TODO: 外部キー情報の実装
        Ok(vec![])
    }

    fn call(&self, _name: &str, _params: &[Value]) -> DbResult<CallResult> {
        Err(DbError::new(
            "Stored procedures not yet implemented for MySQL",
        ))
    }

    fn supports(&self, feature: &str) -> bool {
        matches!(
            feature,
            "transactions" | "prepared_statements" | "stored_procedures"
        )
    }

    fn driver_info(&self) -> DbResult<DriverInfo> {
        let version_sql = "SELECT VERSION() as version";
        let rows = self.query(version_sql, &[], &QueryOptions::default())?;

        let db_version = rows
            .first()
            .and_then(|row| row.get("version"))
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(DriverInfo {
            name: "MySQL".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_version: db_version,
        })
    }

    fn query_info(&self, _sql: &str) -> DbResult<QueryInfo> {
        // TODO: PREPARE文を使ってカラム情報を取得
        Err(DbError::new("Query info not yet implemented for MySQL"))
    }
}

/// MySQLトランザクション
pub struct MysqlTransaction {
    conn: Arc<Mutex<MyConn>>,
    runtime: Arc<Mutex<tokio::runtime::Runtime>>,
    committed: Mutex<bool>,
}

impl DbTransaction for MysqlTransaction {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        // パラメータを変換
        let mysql_params: Vec<mysql_async::Value> = params
            .iter()
            .map(MysqlConnection::value_to_mysql_value)
            .collect();

        let rows: Vec<MyRow> = runtime
            .block_on(async {
                if mysql_params.is_empty() {
                    conn.query(sql).await
                } else {
                    conn.exec(sql, mysql_params).await
                }
            })
            .map_err(|e| {
                DbError::new(fmt_msg(MsgKey::DbFailedToExecuteQuery, &[&e.to_string()]))
            })?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(MysqlConnection::row_to_hashmap(row)?);
        }

        Ok(results)
    }

    fn exec(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<i64> {
        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        // パラメータを変換
        let mysql_params: Vec<mysql_async::Value> = params
            .iter()
            .map(MysqlConnection::value_to_mysql_value)
            .collect();

        let affected = runtime
            .block_on(async {
                if mysql_params.is_empty() {
                    conn.query_drop(sql).await?;
                } else {
                    conn.exec_drop(sql, mysql_params).await?;
                }
                Ok::<_, mysql_async::Error>(conn.affected_rows())
            })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToExecuteStatement,
                    &[&e.to_string()],
                ))
            })?;

        Ok(affected as i64)
    }

    fn commit(self: Arc<Self>) -> DbResult<()> {
        let mut committed = self.committed.lock();
        if *committed {
            return Err(DbError::new("Transaction already committed or rolled back"));
        }

        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        runtime
            .block_on(async { conn.query_drop("COMMIT").await })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToCommitTransaction,
                    &[&e.to_string()],
                ))
            })?;

        *committed = true;
        Ok(())
    }

    fn rollback(self: Arc<Self>) -> DbResult<()> {
        let mut committed = self.committed.lock();
        if *committed {
            return Err(DbError::new("Transaction already committed or rolled back"));
        }

        let mut conn = self.conn.lock();
        let runtime = self.runtime.lock();

        runtime
            .block_on(async { conn.query_drop("ROLLBACK").await })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToRollbackTransaction,
                    &[&e.to_string()],
                ))
            })?;

        *committed = true;

        Ok(())
    }
}

// ヘルパー拡張トレイト
trait ValueExt {
    fn as_string(&self) -> Option<String>;
}

impl ValueExt for Value {
    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}
