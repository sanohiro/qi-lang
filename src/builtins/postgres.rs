//! PostgreSQLドライバー実装
//!
//! PostgreSQLデータベース接続を提供。
//! - リモート接続対応
//! - ACID準拠のトランザクション
//! - プリペアドステートメント
//!
//! このモジュールは `db-postgres` feature でコンパイルされます。

#![cfg(feature = "db-postgres")]

use super::db::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls, Row as PgRow};

/// PostgreSQLドライバー
pub struct PostgresDriver;

impl Default for PostgresDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self
    }
}

impl DbDriver for PostgresDriver {
    fn connect(&self, url: &str, _opts: &ConnectionOptions) -> DbResult<Arc<dyn DbConnection>> {
        // URL形式: "postgres://user:pass@host:port/dbname" または "postgresql://..."
        let conn_str = if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            url
        } else {
            return Err(DbError::new(
                "Invalid PostgreSQL URL. Expected format: postgres://user:pass@host:port/dbname",
            ));
        };

        // tokioランタイムを作成
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DbError::new(format!("Failed to create runtime: {}", e)))?;

        // 接続を確立
        let (client, connection) = rt
            .block_on(async { tokio_postgres::connect(conn_str, NoTls).await })
            .map_err(|e| DbError::new(fmt_msg(MsgKey::DbFailedToConnect, &[&e.to_string()])))?;

        // 接続ハンドラーをバックグラウンドで実行
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = connection.await {
                    eprintln!("PostgreSQL connection error: {}", e);
                }
            });
        });

        Ok(Arc::new(PostgresConnection {
            client: Arc::new(Mutex::new(client)),
            runtime: Arc::new(Mutex::new(rt)),
        }))
    }

    fn name(&self) -> &str {
        "postgres"
    }
}

/// PostgreSQL接続
pub struct PostgresConnection {
    client: Arc<Mutex<Client>>,
    runtime: Arc<Mutex<tokio::runtime::Runtime>>,
}

impl PostgresConnection {
    /// Valueをtokio_postgresのパラメータに変換
    fn value_to_param_string(value: &Value) -> String {
        match value {
            Value::Nil => "NULL".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => format!("'{}'", s.replace('\'', "''")),
            _ => format!("'{}'", value.to_string().replace('\'', "''")),
        }
    }

    /// PostgreSQL行をQi Valueに変換
    fn row_to_hashmap(row: &PgRow) -> DbResult<Row> {
        let mut map = Row::new();

        for (idx, column) in row.columns().iter().enumerate() {
            let column_name = column.name().to_string();

            // 型に応じて値を取得
            let value = if let Ok(v) = row.try_get::<_, Option<String>>(idx) {
                v.map(Value::String).unwrap_or(Value::Nil)
            } else if let Ok(v) = row.try_get::<_, Option<i32>>(idx) {
                v.map(|i| Value::Integer(i as i64)).unwrap_or(Value::Nil)
            } else if let Ok(v) = row.try_get::<_, Option<i64>>(idx) {
                v.map(Value::Integer).unwrap_or(Value::Nil)
            } else if let Ok(v) = row.try_get::<_, Option<f64>>(idx) {
                v.map(Value::Float).unwrap_or(Value::Nil)
            } else if let Ok(v) = row.try_get::<_, Option<bool>>(idx) {
                v.map(Value::Bool).unwrap_or(Value::Nil)
            } else {
                // サポートされていない型
                Value::Nil
            };

            map.insert(column_name, value);
        }

        Ok(map)
    }
}

impl DbConnection for PostgresConnection {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let client = self.client.lock();
        let runtime = self.runtime.lock();

        // パラメータを文字列に変換（簡易実装）
        let param_strings: Vec<String> = params.iter().map(Self::value_to_param_string).collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = param_strings
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = runtime
            .block_on(async { client.query(sql, &param_refs[..]).await })
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
        let client = self.client.lock();
        let runtime = self.runtime.lock();

        // パラメータを文字列に変換（簡易実装）
        let param_strings: Vec<String> = params.iter().map(Self::value_to_param_string).collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = param_strings
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let affected = runtime
            .block_on(async { client.execute(sql, &param_refs[..]).await })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToExecuteStatement,
                    &[&e.to_string()],
                ))
            })?;

        Ok(affected as i64)
    }

    fn begin(&self, _opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>> {
        let client = self.client.lock();
        let runtime = self.runtime.lock();

        // トランザクション開始
        runtime
            .block_on(async { client.execute("BEGIN", &[]).await })
            .map_err(|e| {
                DbError::new(fmt_msg(
                    MsgKey::DbFailedToBeginTransaction,
                    &[&e.to_string()],
                ))
            })?;

        Ok(Arc::new(PostgresTransaction {
            client: self.client.clone(),
            runtime: self.runtime.clone(),
            committed: Mutex::new(false),
        }))
    }

    fn close(&self) -> DbResult<()> {
        // PostgreSQLのクライアントは自動的にドロップ時にクリーンアップされる
        Ok(())
    }

    fn sanitize(&self, value: &str) -> String {
        // シングルクォートをエスケープ
        value.replace('\'', "''")
    }

    fn sanitize_identifier(&self, name: &str) -> String {
        // ダブルクォートで囲む（PostgreSQL識別子）
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn escape_like(&self, pattern: &str) -> String {
        // LIKE句のメタキャラクタをエスケープ
        pattern
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }

    fn driver_name(&self) -> &str {
        "postgres"
    }

    fn tables(&self) -> DbResult<Vec<String>> {
        let sql = "SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname NOT IN ('pg_catalog', 'information_schema') ORDER BY tablename";
        let rows = self.query(sql, &[], &QueryOptions::default())?;

        let tables = rows
            .into_iter()
            .filter_map(|row| {
                row.get("tablename").and_then(|v| {
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
        let sql = "SELECT column_name, data_type, is_nullable, column_default \
                   FROM information_schema.columns \
                   WHERE table_name = $1 \
                   ORDER BY ordinal_position";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let columns = rows
            .into_iter()
            .filter_map(|row| {
                let name = row.get("column_name")?.as_string()?;
                let data_type = row.get("data_type")?.as_string()?;
                let nullable = row.get("is_nullable")?.as_string()? == "YES";
                let default_value = row.get("column_default").and_then(|v| v.as_string());

                Some(ColumnInfo {
                    name,
                    data_type,
                    nullable,
                    default_value,
                    primary_key: false, // TODO: 主キー情報を取得
                })
            })
            .collect();

        Ok(columns)
    }

    fn indexes(&self, table: &str) -> DbResult<Vec<IndexInfo>> {
        let sql = "SELECT indexname, indexdef \
                   FROM pg_indexes \
                   WHERE tablename = $1";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let indexes = rows
            .into_iter()
            .filter_map(|row| {
                let name = row.get("indexname")?.as_string()?;

                Some(IndexInfo {
                    name,
                    table: table.to_string(),
                    columns: vec![], // TODO: カラム情報を取得
                    unique: false,   // TODO: ユニーク制約を取得
                })
            })
            .collect();

        Ok(indexes)
    }

    fn foreign_keys(&self, _table: &str) -> DbResult<Vec<ForeignKeyInfo>> {
        // TODO: 外部キー情報の実装
        Ok(vec![])
    }

    fn call(&self, _name: &str, _params: &[Value]) -> DbResult<CallResult> {
        Err(DbError::new(
            "Stored procedures not yet implemented for PostgreSQL",
        ))
    }

    fn supports(&self, feature: &str) -> bool {
        matches!(
            feature,
            "transactions" | "prepared_statements" | "stored_procedures"
        )
    }

    fn driver_info(&self) -> DbResult<DriverInfo> {
        let version_sql = "SELECT version()";
        let rows = self.query(version_sql, &[], &QueryOptions::default())?;

        let db_version = rows
            .first()
            .and_then(|row| row.get("version"))
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(DriverInfo {
            name: "PostgreSQL".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_version: db_version,
        })
    }

    fn query_info(&self, _sql: &str) -> DbResult<QueryInfo> {
        // TODO: PREPARE文を使ってカラム情報を取得
        Err(DbError::new(
            "Query info not yet implemented for PostgreSQL",
        ))
    }
}

/// PostgreSQLトランザクション
pub struct PostgresTransaction {
    client: Arc<Mutex<Client>>,
    runtime: Arc<Mutex<tokio::runtime::Runtime>>,
    committed: Mutex<bool>,
}

impl DbTransaction for PostgresTransaction {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let client = self.client.lock();
        let runtime = self.runtime.lock();

        // パラメータを文字列に変換（簡易実装）
        let param_strings: Vec<String> = params
            .iter()
            .map(PostgresConnection::value_to_param_string)
            .collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = param_strings
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = runtime
            .block_on(async { client.query(sql, &param_refs[..]).await })
            .map_err(|e| {
                DbError::new(fmt_msg(MsgKey::DbFailedToExecuteQuery, &[&e.to_string()]))
            })?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(PostgresConnection::row_to_hashmap(row)?);
        }

        Ok(results)
    }

    fn exec(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<i64> {
        let client = self.client.lock();
        let runtime = self.runtime.lock();

        // パラメータを文字列に変換（簡易実装）
        let param_strings: Vec<String> = params
            .iter()
            .map(PostgresConnection::value_to_param_string)
            .collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = param_strings
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let affected = runtime
            .block_on(async { client.execute(sql, &param_refs[..]).await })
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

        let client = self.client.lock();
        let runtime = self.runtime.lock();

        runtime
            .block_on(async { client.execute("COMMIT", &[]).await })
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

        let client = self.client.lock();
        let runtime = self.runtime.lock();

        runtime
            .block_on(async { client.execute("ROLLBACK", &[]).await })
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
