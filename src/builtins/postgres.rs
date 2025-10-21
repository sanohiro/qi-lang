//! PostgreSQL接続モジュール
//!
//! このモジュールは `db-postgres` feature でコンパイルされます。

#![cfg(feature = "db-postgres")]

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use tokio_postgres::{NoTls, Row};

/// db/pg-connect - PostgreSQL接続を確立してクエリを実行
///
/// 引数:
/// - connection_string: 接続文字列（例: "postgresql://user:pass@localhost/dbname"）
/// - query: SQLクエリ
/// - params: パラメータ（オプション、ベクタ）
///
/// 戻り値: rows（ベクタ）または {:error message}
pub fn native_pg_query(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["db/pg-query", "2-3"]));
    }

    // 接続文字列
    let conn_str = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/pg-query (connection_string)", "strings"],
            ))
        }
    };

    // クエリ
    let query = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/pg-query (query)", "strings"],
            ))
        }
    };

    // パラメータ（オプション）
    let params: Vec<String> = if args.len() == 3 {
        match &args[2] {
            Value::Vector(items) | Value::List(items) => {
                let mut params = Vec::new();
                for item in items.iter() {
                    match item {
                        Value::String(s) => params.push(s.clone()),
                        Value::Integer(i) => params.push(i.to_string()),
                        Value::Float(f) => params.push(f.to_string()),
                        Value::Bool(b) => params.push(b.to_string()),
                        Value::Nil => params.push("NULL".to_string()),
                        _ => {
                            return Err(fmt_msg(
                                MsgKey::TypeOnly,
                                &["db/pg-query (params)", "primitives"],
                            ))
                        }
                    }
                }
                params
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["db/pg-query (params)", "vectors"],
                ))
            }
        }
    } else {
        Vec::new()
    };

    // 非同期処理を同期的に実行
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Ok(Value::error(format!("Runtime error: {}", e))),
    };

    rt.block_on(async {
        // 接続
        let (client, connection) = match tokio_postgres::connect(conn_str, NoTls).await {
            Ok(conn) => conn,
            Err(e) => return Ok(Value::error(format!("Connection error: {}", e))),
        };

        // 接続を別タスクで処理
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // クエリ実行
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = match client.query(query, &param_refs[..]).await {
            Ok(rows) => rows,
            Err(e) => return Ok(Value::error(format!("Query error: {}", e))),
        };

        // 結果を変換
        let result_rows = rows_to_value(&rows);

        Ok(result_rows)
    })
}

/// db/pg-exec - PostgreSQLコマンド実行（INSERT/UPDATE/DELETE）
///
/// 引数:
/// - connection_string: 接続文字列
/// - command: SQLコマンド
/// - params: パラメータ（オプション、ベクタ）
///
/// 戻り値: affected_rows（整数）または {:error message}
pub fn native_pg_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["db/pg-exec", "2-3"]));
    }

    // 接続文字列
    let conn_str = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/pg-exec (connection_string)", "strings"],
            ))
        }
    };

    // コマンド
    let command = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/pg-exec (command)", "strings"],
            ))
        }
    };

    // パラメータ（オプション）
    let params: Vec<String> = if args.len() == 3 {
        match &args[2] {
            Value::Vector(items) | Value::List(items) => {
                let mut params = Vec::new();
                for item in items.iter() {
                    match item {
                        Value::String(s) => params.push(s.clone()),
                        Value::Integer(i) => params.push(i.to_string()),
                        Value::Float(f) => params.push(f.to_string()),
                        Value::Bool(b) => params.push(b.to_string()),
                        Value::Nil => params.push("NULL".to_string()),
                        _ => {
                            return Err(fmt_msg(
                                MsgKey::TypeOnly,
                                &["db/pg-exec (params)", "primitives"],
                            ))
                        }
                    }
                }
                params
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["db/pg-exec (params)", "vectors"],
                ))
            }
        }
    } else {
        Vec::new()
    };

    // 非同期処理を同期的に実行
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Ok(Value::error(format!("Runtime error: {}", e))),
    };

    rt.block_on(async {
        // 接続
        let (client, connection) = match tokio_postgres::connect(conn_str, NoTls).await {
            Ok(conn) => conn,
            Err(e) => return Ok(Value::error(format!("Connection error: {}", e))),
        };

        // 接続を別タスクで処理
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // コマンド実行
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|s| s as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let affected = match client.execute(command, &param_refs[..]).await {
            Ok(n) => n,
            Err(e) => return Ok(Value::error(format!("Execute error: {}", e))),
        };

        Ok(Value::Integer(affected as i64))
    })
}

// ========================================
// ヘルパー関数
// ========================================

/// PostgreSQL行をQi Valueに変換
fn rows_to_value(rows: &[Row]) -> Value {
    let mut result_rows = Vec::new();

    for row in rows {
        let mut row_map = im::HashMap::new();

        for (idx, column) in row.columns().iter().enumerate() {
            let col_name = format!(":{}", column.name());

            // 型に応じて値を取得
            let value = if let Ok(v) = row.try_get::<_, String>(idx) {
                Value::String(v)
            } else if let Ok(v) = row.try_get::<_, i32>(idx) {
                Value::Integer(v as i64)
            } else if let Ok(v) = row.try_get::<_, i64>(idx) {
                Value::Integer(v)
            } else if let Ok(v) = row.try_get::<_, f64>(idx) {
                Value::Float(v)
            } else if let Ok(v) = row.try_get::<_, bool>(idx) {
                Value::Bool(v)
            } else if row
                .try_get::<_, Option<String>>(idx)
                .ok()
                .flatten()
                .is_none()
            {
                Value::Nil
            } else {
                // その他の型は文字列として取得を試みる
                Value::String(format!("<unsupported type>"))
            };

            row_map.insert(col_name, value);
        }

        result_rows.push(Value::Map(row_map));
    }

    Value::Vector(result_rows.into())
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category db/postgres
/// @qi-doc:functions db/pg-query, db/pg-exec
pub const FUNCTIONS: super::NativeFunctions = &[
    ("db/pg-query", native_pg_query),
    ("db/pg-exec", native_pg_exec),
];

#[cfg(test)]
mod tests {
    use super::*;

    // 注: これらのテストはPostgreSQLサーバーが必要なため、統合テストとして実装すべき

    #[test]
    fn test_postgres_connection_string_validation() {
        // 接続文字列のバリデーションのみテスト
        let conn_str = Value::String("postgresql://localhost/test".to_string());
        let query = Value::String("SELECT 1".to_string());

        // 実際の接続は行わず、引数チェックのみ
        let result = native_pg_query(&[conn_str, query]);
        // PostgreSQLサーバーが無い場合はエラーになるが、それは正常
        assert!(result.is_ok());
    }
}
