use super::*;
use crate::builtins::db::types::*;
use crate::i18n::{fmt_msg, MsgKey};

/// SQLクエリを実行してすべての結果行を取得する
///
/// # 引数
/// - `conn_id` (DbConnection | DbTransaction): 接続またはトランザクション
/// - `sql` (string): SQL文
/// - `params` (vector, optional): バインドパラメータ
/// - `options` (map, optional): クエリオプション
///
/// # 戻り値
/// - (vector): 結果行のベクトル (各行はマップ)
///
/// # 例
/// ```qi
/// (let rows (db/query conn "SELECT * FROM users WHERE age > ?" [18]))
/// ```
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
            // 接続をクローンしてからミューテックスを解放（デッドロック回避）
            let conn = {
                let connections = CONNECTIONS.lock();
                connections
                    .get(&conn_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
                    .clone()
            };
            conn.query(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            // トランザクションをクローンしてからミューテックスを解放
            let tx = {
                let transactions = TRANSACTIONS.lock();
                transactions
                    .get(&tx_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?
                    .clone()
            };
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

    // 接続かトランザクションかを判別
    let row = match extract_conn_or_tx(&args[0])? {
        ConnOrTx::Conn(conn_id) => {
            // 接続をクローンしてからミューテックスを解放
            let conn = {
                let connections = CONNECTIONS.lock();
                connections
                    .get(&conn_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
                    .clone()
            };
            conn.query_one(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            // トランザクションをクローンしてからミューテックスを解放
            let tx = {
                let transactions = TRANSACTIONS.lock();
                transactions
                    .get(&tx_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?
                    .clone()
            };
            tx.query_one(sql, &params, &opts).map_err(|e| e.message)?
        }
    };

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
            // 接続をクローンしてからミューテックスを解放（デッドロック回避）
            let conn = {
                let connections = CONNECTIONS.lock();
                connections
                    .get(&conn_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
                    .clone()
            };
            conn.exec(sql, &params, &opts).map_err(|e| e.message)?
        }
        ConnOrTx::Tx(tx_id) => {
            // トランザクションをクローンしてからミューテックスを解放
            let tx = {
                let transactions = TRANSACTIONS.lock();
                transactions
                    .get(&tx_id)
                    .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?
                    .clone()
            };
            tx.exec(sql, &params, &opts).map_err(|e| e.message)?
        }
    };

    Ok(Value::Integer(affected))
}
