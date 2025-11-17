use super::*;
use crate::builtins::db::types::*;
use crate::i18n::{fmt_msg, MsgKey};

pub fn native_begin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["db/begin"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let opts = if args.len() == 2 {
        TransactionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        TransactionOptions::default()
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let tx = conn.begin(&opts).map_err(|e| e.message)?;

    // トランザクションを保存
    let tx_id = gen_tx_id();
    TRANSACTIONS.lock().insert(tx_id.clone(), tx);

    Ok(Value::String(format!("DbTransaction:{}", tx_id)))
}

/// db/commit - トランザクションをコミット
pub fn native_commit(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/commit"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    let mut transactions = TRANSACTIONS.lock();
    let tx = transactions
        .remove(&tx_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;

    tx.commit().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

/// db/rollback - トランザクションをロールバック
pub fn native_rollback(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/rollback"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    let mut transactions = TRANSACTIONS.lock();
    let tx = transactions
        .remove(&tx_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?;

    tx.rollback().map_err(|e| e.message)?;

    Ok(Value::Nil)
}
