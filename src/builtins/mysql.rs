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
            .map_err(|e| DbError::new(fmt_msg(MsgKey::FailedToCreateRuntime, &[&e.to_string()])))?;

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
    ///
    /// カラムの型情報とValueを直接確認することで、試行錯誤なしに確実な型変換を実現
    fn row_to_hashmap(row: &MyRow) -> DbResult<Row> {
        use base64::{engine::general_purpose, Engine as _};
        use mysql_async::Value as MySqlValue;

        let mut map = Row::new();
        let columns = row.columns_ref();

        for (idx, column) in columns.iter().enumerate() {
            let column_name = column.name_str().to_string();
            let column_type = column.column_type();

            // カラムの値を直接取得
            let mysql_value = row.as_ref(idx).unwrap_or(&MySqlValue::NULL);

            // MySQLのValueを直接パターンマッチで変換（型情報に基づく確実な変換）
            let value = match mysql_value {
                MySqlValue::NULL => Value::Nil,
                MySqlValue::Int(i) => Value::Integer(*i),
                MySqlValue::UInt(u) => {
                    // u64 -> i64の安全な変換（Qiはi64のみサポート）
                    if *u <= i64::MAX as u64 {
                        Value::Integer(*u as i64)
                    } else {
                        Value::String(u.to_string())
                    }
                }
                MySqlValue::Float(f) => Value::Float(*f as f64),
                MySqlValue::Double(d) => Value::Float(*d),
                MySqlValue::Bytes(b) => {
                    // カラムの型情報を使って適切に変換
                    if column_type.is_numeric_type() {
                        // 数値型の場合はパース
                        if let Ok(s) = String::from_utf8(b.clone()) {
                            if let Ok(i) = s.parse::<i64>() {
                                Value::Integer(i)
                            } else if let Ok(f) = s.parse::<f64>() {
                                Value::Float(f)
                            } else {
                                Value::String(s)
                            }
                        } else {
                            Value::Nil
                        }
                    } else {
                        // 文字列型またはバイナリデータ
                        String::from_utf8(b.clone())
                            .map(Value::String)
                            .unwrap_or_else(|_| {
                                // UTF-8でない場合はbase64エンコード
                                Value::String(format!(
                                    "base64:{}",
                                    general_purpose::STANDARD.encode(b)
                                ))
                            })
                    }
                }
                MySqlValue::Date(year, month, day, hour, min, sec, _micro) => {
                    // 日付時刻をISO 8601形式の文字列に変換
                    Value::String(format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                        year, month, day, hour, min, sec
                    ))
                }
                MySqlValue::Time(is_neg, days, hours, mins, secs, _micro) => {
                    // 時間をHH:MM:SS形式の文字列に変換
                    let total_hours = days * 24 + *hours as u32;
                    Value::String(format!(
                        "{}{:02}:{:02}:{:02}",
                        if *is_neg { "-" } else { "" },
                        total_hours,
                        mins,
                        secs
                    ))
                }
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

    fn foreign_keys(&self, table: &str) -> DbResult<Vec<ForeignKeyInfo>> {
        // 外部キー情報を取得
        let sql = "SELECT \
                       kcu.CONSTRAINT_NAME as constraint_name, \
                       kcu.COLUMN_NAME as column_name, \
                       kcu.REFERENCED_TABLE_NAME as referenced_table, \
                       kcu.REFERENCED_COLUMN_NAME as referenced_column, \
                       rc.UPDATE_RULE as update_rule, \
                       rc.DELETE_RULE as delete_rule \
                   FROM information_schema.KEY_COLUMN_USAGE kcu \
                   JOIN information_schema.REFERENTIAL_CONSTRAINTS rc \
                       ON kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME \
                       AND kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA \
                   WHERE kcu.TABLE_NAME = ? \
                       AND kcu.REFERENCED_TABLE_NAME IS NOT NULL \
                   ORDER BY kcu.ORDINAL_POSITION";

        let rows = self.query(
            sql,
            &[Value::String(table.to_string())],
            &QueryOptions::default(),
        )?;

        let foreign_keys = rows
            .into_iter()
            .filter_map(|row| {
                let name = row.get("constraint_name")?.as_string()?;
                let column = row.get("column_name")?.as_string()?;
                let referenced_table = row.get("referenced_table")?.as_string()?;
                let referenced_column = row.get("referenced_column")?.as_string()?;
                let on_update = row.get("update_rule").and_then(|v| v.as_string());
                let on_delete = row.get("delete_rule").and_then(|v| v.as_string());

                Some(ForeignKeyInfo {
                    name,
                    table: table.to_string(),
                    column,
                    referenced_table,
                    referenced_column,
                    on_update,
                    on_delete,
                })
            })
            .collect();

        Ok(foreign_keys)
    }

    fn call(&self, name: &str, params: &[Value]) -> DbResult<CallResult> {
        // MySQLではCALLでプロシージャを実行
        let placeholders: Vec<String> = params.iter().map(|_| "?".to_string()).collect();
        let call_sql = format!("CALL {}({})", name, placeholders.join(", "));

        let mut conn = self.conn.lock();
        let mysql_params = params_to_mysql(params);

        // CALLを実行
        let result = conn
            .exec_iter(&call_sql, mysql_params)
            .map_err(|e| DbError::new(&format!("Failed to call procedure: {}", e)))?;

        // 結果セットを収集
        let mut all_rows = Vec::new();

        for result_set in result {
            let result_set =
                result_set.map_err(|e| DbError::new(&format!("Failed to fetch results: {}", e)))?;

            let rows: Result<Vec<Row>, _> = result_set
                .map(|row| {
                    row.map_err(|e| DbError::new(&format!("Failed to read row: {}", e)))
                        .and_then(mysql_row_to_row)
                })
                .collect();

            all_rows.push(rows?);
        }

        // 結果の形式を判定
        if all_rows.is_empty() {
            Ok(CallResult::Value(Value::Nil))
        } else if all_rows.len() == 1 {
            Ok(CallResult::Rows(all_rows.into_iter().next().unwrap()))
        } else {
            Ok(CallResult::Multiple(all_rows))
        }
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

    fn query_info(&self, sql: &str) -> DbResult<QueryInfo> {
        // PREPARE文を使ってクエリのメタ情報を取得
        let stmt_name = format!(
            "qi_query_info_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let mut conn = self.conn.lock();

        // PREPARE文でクエリを準備
        let prepare_sql = format!("PREPARE {} FROM '{}'", stmt_name, sql.replace("'", "''"));
        conn.query_drop(&prepare_sql)
            .map_err(|e| DbError::new(&format!("Failed to prepare query: {}", e)))?;

        // EXPLAIN文でカラム情報を推測（MySQLにはDESCRIBE PREPAREがないため）
        // 注: これは完全な解決策ではなく、SELECT文のみ対応
        let explain_sql = format!("EXPLAIN {}", sql);
        let result = conn.query_map(&explain_sql, |_: mysql::Row| ());

        // PREPARE文をクリーンアップ
        let deallocate_sql = format!("DEALLOCATE PREPARE {}", stmt_name);
        let _ = conn.query_drop(&deallocate_sql);

        // MySQLの場合、カラム情報を正確に取得するのは困難なため、
        // 空のリストとパラメータ数0を返す
        match result {
            Ok(_) => Ok(QueryInfo {
                columns: vec![],
                param_count: 0,
            }),
            Err(e) => Err(DbError::new(&format!("Failed to analyze query: {}", e))),
        }
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
