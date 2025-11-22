//! PostgreSQLドライバー実装
//!
//! PostgreSQLデータベース接続を提供。
//! - リモート接続対応
//! - ACID準拠のトランザクション
//! - プリペアドステートメント
//!
//! このモジュールは `db-postgres` feature でコンパイルされます。

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

        // 共有PostgreSQLランタイムを取得
        let rt = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // 接続を確立
        let (client, connection) = rt
            .block_on(async { tokio_postgres::connect(conn_str, NoTls).await })
            .map_err(|e| DbError::new(fmt_msg(MsgKey::DbFailedToConnect, &[&e.to_string()])))?;

        // 接続ハンドラーをバックグラウンドタスクとして実行（共有ランタイム上で）
        rt.spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(Arc::new(PostgresConnection {
            client: Arc::new(Mutex::new(client)),
        }))
    }

    fn name(&self) -> &str {
        "postgres"
    }
}

/// PostgreSQL接続
pub struct PostgresConnection {
    client: Arc<Mutex<Client>>,
}

impl PostgresConnection {
    /// Valueをプリペアドステートメント用のパラメータに変換
    /// 注: プリペアドステートメントではエスケープ不要（DBドライバーが自動処理）
    fn value_to_sql_param(value: &Value) -> Box<dyn tokio_postgres::types::ToSql + Sync + Send> {
        match value {
            Value::Nil => Box::new(None::<String>),
            Value::Bool(b) => Box::new(*b),
            Value::Integer(i) => Box::new(*i), // i64のまま送る（PostgreSQL BIGINT型）
            Value::Float(f) => Box::new(*f),
            Value::String(s) => Box::new(s.clone()),
            Value::Bytes(b) => Box::new(b.as_ref().to_vec()), // BYTEA型として送信
            _ => Box::new(value.to_string()),
        }
    }

    /// PostgreSQL行をQi Valueに変換
    fn row_to_hashmap(row: &PgRow) -> DbResult<Row> {
        let mut map = crate::new_hashmap();

        for (idx, column) in row.columns().iter().enumerate() {
            let column_name = column.name().to_string();

            // 型に応じて値を取得
            let value = if let Ok(v) = row.try_get::<_, Option<Vec<u8>>>(idx) {
                // BYTEA型 → Bytes
                v.map(|bytes| Value::Bytes(Arc::from(bytes)))
                    .unwrap_or(Value::Nil)
            } else if let Ok(v) = row.try_get::<_, Option<String>>(idx) {
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

            map.insert(crate::value::MapKey::String(column_name), value);
        }

        Ok(map)
    }
}

impl DbConnection for PostgresConnection {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let client = self.client.lock();
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // Valueをプリペアドステートメント用パラメータに変換
        let sql_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            params.iter().map(Self::value_to_sql_param).collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = sql_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
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
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // Valueをプリペアドステートメント用パラメータに変換
        let sql_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            params.iter().map(Self::value_to_sql_param).collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = sql_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
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
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

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
                row.get(&crate::value::MapKey::String("tablename".to_string()))
                    .and_then(|v| {
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
        // カラム情報と主キー情報を同時に取得
        let sql = "SELECT c.column_name, c.data_type, c.is_nullable, c.column_default, \
                          CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key \
                   FROM information_schema.columns c \
                   LEFT JOIN ( \
                       SELECT ku.column_name \
                       FROM information_schema.table_constraints tc \
                       JOIN information_schema.key_column_usage ku \
                           ON tc.constraint_name = ku.constraint_name \
                       WHERE tc.constraint_type = 'PRIMARY KEY' \
                           AND tc.table_name = $1 \
                   ) pk ON c.column_name = pk.column_name \
                   WHERE c.table_name = $1 \
                   ORDER BY c.ordinal_position";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let columns = rows
            .into_iter()
            .filter_map(|row| {
                let name = row
                    .get(&crate::value::MapKey::String("column_name".to_string()))?
                    .as_string()?;
                let data_type = row
                    .get(&crate::value::MapKey::String("data_type".to_string()))?
                    .as_string()?;
                let nullable = row
                    .get(&crate::value::MapKey::String("is_nullable".to_string()))?
                    .as_string()?
                    == "YES";
                let default_value = row
                    .get(&crate::value::MapKey::String("column_default".to_string()))
                    .and_then(|v| v.as_string());
                let primary_key = row
                    .get(&crate::value::MapKey::String("is_primary_key".to_string()))
                    .and_then(|v| match v {
                        Value::Bool(b) => Some(*b),
                        _ => None,
                    })
                    .unwrap_or(false);

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
        // インデックス情報、カラム、ユニーク制約を取得
        let sql = "SELECT i.indexname, \
                          ix.indisunique, \
                          array_agg(a.attname ORDER BY ak.attnum) as column_names \
                   FROM pg_indexes i \
                   JOIN pg_class c ON c.relname = i.tablename \
                   JOIN pg_index ix ON ix.indrelid = c.oid \
                   JOIN pg_class ci ON ci.oid = ix.indexrelid \
                   JOIN pg_attribute ak ON ak.attrelid = ix.indexrelid \
                   JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = ANY(ix.indkey) \
                   WHERE i.tablename = $1 AND i.indexname = ci.relname \
                   GROUP BY i.indexname, ix.indisunique";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let indexes = rows
            .into_iter()
            .filter_map(|row| {
                let name = row
                    .get(&crate::value::MapKey::String("indexname".to_string()))?
                    .as_string()?;
                let unique = row
                    .get(&crate::value::MapKey::String("indisunique".to_string()))
                    .and_then(|v| match v {
                        Value::Bool(b) => Some(*b),
                        _ => None,
                    })
                    .unwrap_or(false);

                // PostgreSQLの配列型から文字列のベクタに変換
                let columns = if let Some(Value::String(col_str)) =
                    row.get(&crate::value::MapKey::String("column_names".to_string()))
                {
                    // PostgreSQLの配列は "{col1,col2}" の形式なので "{" と "}" を除去して分割
                    col_str
                        .trim_matches(|c| c == '{' || c == '}')
                        .split(',')
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![]
                };

                Some(IndexInfo {
                    name,
                    table: table.to_string(),
                    columns,
                    unique,
                })
            })
            .collect();

        Ok(indexes)
    }

    fn foreign_keys(&self, table: &str) -> DbResult<Vec<ForeignKeyInfo>> {
        // 外部キー情報を取得
        let sql = "SELECT \
                       tc.constraint_name, \
                       kcu.column_name, \
                       ccu.table_name AS referenced_table, \
                       ccu.column_name AS referenced_column, \
                       rc.update_rule, \
                       rc.delete_rule \
                   FROM information_schema.table_constraints AS tc \
                   JOIN information_schema.key_column_usage AS kcu \
                       ON tc.constraint_name = kcu.constraint_name \
                   JOIN information_schema.constraint_column_usage AS ccu \
                       ON ccu.constraint_name = tc.constraint_name \
                   JOIN information_schema.referential_constraints AS rc \
                       ON rc.constraint_name = tc.constraint_name \
                   WHERE tc.constraint_type = 'FOREIGN KEY' \
                       AND tc.table_name = $1";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let foreign_keys = rows
            .into_iter()
            .filter_map(|row| {
                let name = row
                    .get(&crate::value::MapKey::String("constraint_name".to_string()))?
                    .as_string()?;
                let column = row
                    .get(&crate::value::MapKey::String("column_name".to_string()))?
                    .as_string()?;
                let referenced_table = row
                    .get(&crate::value::MapKey::String(
                        "referenced_table".to_string(),
                    ))?
                    .as_string()?;
                let referenced_column = row
                    .get(&crate::value::MapKey::String(
                        "referenced_column".to_string(),
                    ))?
                    .as_string()?;
                let _on_update = row
                    .get(&crate::value::MapKey::String("update_rule".to_string()))
                    .and_then(|v| v.as_string());
                let _on_delete = row
                    .get(&crate::value::MapKey::String("delete_rule".to_string()))
                    .and_then(|v| v.as_string());

                Some(ForeignKeyInfo {
                    name,
                    table: table.to_string(),
                    columns: vec![column],
                    referenced_table,
                    referenced_columns: vec![referenced_column],
                })
            })
            .collect();

        Ok(foreign_keys)
    }

    fn call(&self, name: &str, params: &[Value]) -> DbResult<CallResult> {
        // PostgreSQLではSELECTで関数を呼び出すかCALLでプロシージャを実行
        // まずSELECTで関数として試行
        let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${}", i)).collect();
        let select_sql = format!("SELECT {}({})", name, placeholders.join(", "));

        match self.query(&select_sql, params, &QueryOptions::default()) {
            Ok(rows) => {
                // 関数の結果を取得
                if rows.is_empty() {
                    Ok(CallResult::Value(Value::Nil))
                } else if rows.len() == 1 && rows[0].len() == 1 {
                    // 単一の戻り値
                    let value = rows[0].values().next().cloned().unwrap_or(Value::Nil);
                    Ok(CallResult::Value(value))
                } else {
                    // 複数行または複数カラムの結果
                    Ok(CallResult::Rows(rows))
                }
            }
            Err(_) => {
                // SELECT失敗時はCALLでプロシージャとして試行
                let call_sql = format!("CALL {}({})", name, placeholders.join(", "));
                self.query(&call_sql, params, &QueryOptions::default())?;
                Ok(CallResult::Value(Value::Nil))
            }
        }
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
            .and_then(|row| row.get(&crate::value::MapKey::String("version".to_string())))
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(DriverInfo {
            name: "PostgreSQL".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_version: db_version,
        })
    }

    fn query_info(&self, sql: &str) -> DbResult<QueryInfo> {
        // PREPARE文を使ってクエリのメタ情報を取得
        let stmt_name = format!(
            "qi_query_info_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let client = self.client.lock();
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // PREPARE文でクエリを準備
        let prepare_sql = format!("PREPARE {} AS {}", stmt_name, sql);
        runtime
            .block_on(client.execute(&prepare_sql, &[]))
            .map_err(|e| DbError::new(format!("Failed to prepare query: {}", e)))?;

        // PREPARE文をクリーンアップ
        let deallocate_sql = format!("DEALLOCATE {}", stmt_name);
        let _ = runtime.block_on(client.execute(&deallocate_sql, &[]));

        // PostgreSQLの場合、PREPARE文からカラム情報を取得するのは複雑なため、
        // 空のカラム情報を返す（実際のクエリ実行時に情報が得られる）
        Ok(QueryInfo {
            columns: vec![],
            parameter_count: 0,
        })
    }
}

/// PostgreSQLトランザクション
pub struct PostgresTransaction {
    client: Arc<Mutex<Client>>,
    committed: Mutex<bool>,
}

impl DbTransaction for PostgresTransaction {
    fn query(&self, sql: &str, params: &[Value], _opts: &QueryOptions) -> DbResult<Rows> {
        let client = self.client.lock();
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // Valueをプリペアドステートメント用パラメータに変換
        let sql_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = params
            .iter()
            .map(PostgresConnection::value_to_sql_param)
            .collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = sql_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
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
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

        // Valueをプリペアドステートメント用パラメータに変換
        let sql_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = params
            .iter()
            .map(PostgresConnection::value_to_sql_param)
            .collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = sql_params
            .iter()
            .map(|b| &**b as &(dyn tokio_postgres::types::ToSql + Sync))
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
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

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
        let runtime = crate::builtins::lazy_init::postgres_runtime::get_runtime()
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e])))?;

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
