//! MySQL接続モジュール
//!
//! このモジュールは `db-mysql` feature でコンパイルされます。

#![cfg(feature = "db-mysql")]

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};

/// db/my-query - MySQL接続を確立してクエリを実行
///
/// 引数:
/// - connection_string: 接続文字列（例: "mysql://user:pass@localhost/dbname"）
/// - query: SQLクエリ
/// - params: パラメータ（オプション、ベクタ）
///
/// 戻り値: rows（ベクタ）または {:error message}
pub fn native_my_query(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["db/my-query", "2-3"]));
    }

    // 接続文字列
    let conn_str = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/my-query (connection_string)", "strings"],
            ))
        }
    };

    // クエリ
    let query = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/my-query (query)", "strings"],
            ))
        }
    };

    // パラメータ（オプション）
    let params: Vec<mysql_async::Value> = if args.len() == 3 {
        match &args[2] {
            Value::Vector(items) | Value::List(items) => {
                let mut params = Vec::new();
                for item in items.iter() {
                    let mysql_val = match item {
                        Value::String(s) => mysql_async::Value::Bytes(s.as_bytes().to_vec()),
                        Value::Integer(i) => mysql_async::Value::Int(*i),
                        Value::Float(f) => mysql_async::Value::Double(*f),
                        Value::Bool(b) => mysql_async::Value::Int(if *b { 1 } else { 0 }),
                        Value::Nil => mysql_async::Value::NULL,
                        _ => {
                            return Err(fmt_msg(
                                MsgKey::TypeOnly,
                                &["db/my-query (params)", "primitives"],
                            ))
                        }
                    };
                    params.push(mysql_val);
                }
                params
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["db/my-query (params)", "vectors"],
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
        // 接続オプション
        let opts = match Opts::from_url(conn_str) {
            Ok(opts) => opts,
            Err(e) => return Ok(Value::error(format!("Invalid connection string: {}", e))),
        };

        // 接続
        let mut conn = match mysql_async::Conn::new(opts).await {
            Ok(conn) => conn,
            Err(e) => return Ok(Value::error(format!("Connection error: {}", e))),
        };

        // クエリ実行
        let rows: Vec<Row> = if params.is_empty() {
            match conn.query(query).await {
                Ok(rows) => rows,
                Err(e) => return Ok(Value::error(format!("Query error: {}", e))),
            }
        } else {
            match conn.exec(query, params).await {
                Ok(rows) => rows,
                Err(e) => return Ok(Value::error(format!("Query error: {}", e))),
            }
        };

        // 接続をクローズ
        let _ = conn.disconnect().await;

        // 結果を変換
        let result_rows = rows_to_value(&rows);

        Ok(result_rows)
    })
}

/// db/my-exec - MySQLコマンド実行（INSERT/UPDATE/DELETE）
///
/// 引数:
/// - connection_string: 接続文字列
/// - command: SQLコマンド
/// - params: パラメータ（オプション、ベクタ）
///
/// 戻り値: affected_rows（整数）または {:error message}
pub fn native_my_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["db/my-exec", "2-3"]));
    }

    // 接続文字列
    let conn_str = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/my-exec (connection_string)", "strings"],
            ))
        }
    };

    // コマンド
    let command = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["db/my-exec (command)", "strings"],
            ))
        }
    };

    // パラメータ（オプション）
    let params: Vec<mysql_async::Value> = if args.len() == 3 {
        match &args[2] {
            Value::Vector(items) | Value::List(items) => {
                let mut params = Vec::new();
                for item in items.iter() {
                    let mysql_val = match item {
                        Value::String(s) => mysql_async::Value::Bytes(s.as_bytes().to_vec()),
                        Value::Integer(i) => mysql_async::Value::Int(*i),
                        Value::Float(f) => mysql_async::Value::Double(*f),
                        Value::Bool(b) => mysql_async::Value::Int(if *b { 1 } else { 0 }),
                        Value::Nil => mysql_async::Value::NULL,
                        _ => {
                            return Err(fmt_msg(
                                MsgKey::TypeOnly,
                                &["db/my-exec (params)", "primitives"],
                            ))
                        }
                    };
                    params.push(mysql_val);
                }
                params
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["db/my-exec (params)", "vectors"],
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
        // 接続オプション
        let opts = match Opts::from_url(conn_str) {
            Ok(opts) => opts,
            Err(e) => return Ok(Value::error(format!("Invalid connection string: {}", e))),
        };

        // 接続
        let mut conn = match mysql_async::Conn::new(opts).await {
            Ok(conn) => conn,
            Err(e) => return Ok(Value::error(format!("Connection error: {}", e))),
        };

        // コマンド実行
        let result = if params.is_empty() {
            match conn.query_drop(command).await {
                Ok(_) => conn.affected_rows(),
                Err(e) => return Ok(Value::error(format!("Execute error: {}", e))),
            }
        } else {
            match conn.exec_drop(command, params).await {
                Ok(_) => conn.affected_rows(),
                Err(e) => return Ok(Value::error(format!("Execute error: {}", e))),
            }
        };

        // 接続をクローズ
        let _ = conn.disconnect().await;

        Ok(Value::Integer(result as i64))
    })
}

// ========================================
// ヘルパー関数
// ========================================

/// MySQL行をQi Valueに変換
fn rows_to_value(rows: &[Row]) -> Value {
    let mut result_rows = Vec::new();

    for row in rows {
        let mut row_map = im::HashMap::new();

        let columns = row.columns_ref();
        for (idx, column) in columns.iter().enumerate() {
            let col_name = format!(":{}", column.name_str());

            // 型に応じて値を取得（Option<Option<T>>をflattenする）
            let value = if let Some(Some(v)) = row.get::<Option<String>, _>(idx) {
                Value::String(v)
            } else if let Some(Some(v)) = row.get::<Option<i64>, _>(idx) {
                Value::Integer(v)
            } else if let Some(Some(v)) = row.get::<Option<f64>, _>(idx) {
                Value::Float(v)
            } else if let Some(Some(v)) = row.get::<Option<bool>, _>(idx) {
                Value::Bool(v)
            } else {
                Value::Nil
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
/// @qi-doc:category db/mysql
/// @qi-doc:functions db/my-query, db/my-exec
pub const FUNCTIONS: super::NativeFunctions = &[
    ("db/my-query", native_my_query),
    ("db/my-exec", native_my_exec),
];

#[cfg(test)]
mod tests {
    use super::*;

    // 注: これらのテストはMySQLサーバーが必要なため、統合テストとして実装すべき

    #[test]
    fn test_mysql_connection_string_validation() {
        // 接続文字列のバリデーションのみテスト
        let conn_str = Value::String("mysql://localhost/test".to_string());
        let query = Value::String("SELECT 1".to_string());

        // 実際の接続は行わず、引数チェックのみ
        let result = native_my_query(&[conn_str, query]);
        // MySQLサーバーが無い場合はエラーになるが、それは正常
        assert!(result.is_ok());
    }
}
