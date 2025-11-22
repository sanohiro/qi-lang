use super::*;
use crate::builtins::db::types::*;
use crate::i18n::{fmt_msg, MsgKey};

/// トランザクションを開始する
///
/// # 引数
/// - `conn_id` (DbConnection): データベース接続
/// - `options` (map, optional): トランザクションオプション (:isolation-level, :read-only など)
///
/// # 戻り値
/// - (DbTransaction): トランザクション
///
/// # 例
/// ```qi
/// (let tx (db/begin conn))
/// (db/query tx "INSERT INTO users (name) VALUES (?)" ["Alice"])
/// (db/commit tx)
/// ```
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

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    // 接続がプールされているか確認
    let pool_info = {
        let pooled_conns = POOLED_CONNECTIONS.lock();
        pooled_conns
            .get(&conn_id)
            .map(|pool_id| (pool_id.clone(), conn_id.clone()))
    };

    let tx = conn.begin(&opts).map_err(|e| e.message)?;

    // トランザクションを保存
    let tx_id = gen_tx_id();
    TRANSACTIONS.lock().insert(tx_id.clone(), tx);

    // プール接続の場合、トランザクション→プールマッピングを保存
    if let Some((pool_id, conn_id)) = pool_info {
        TRANSACTION_POOLS
            .lock()
            .insert(tx_id.clone(), (pool_id, conn_id));
    }

    Ok(Value::String(format!("DbTransaction:{}", tx_id)))
}

/// db/commit - トランザクションをコミット
pub fn native_commit(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/commit"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    // トランザクションをクローンしてからミューテックスを解放（成功するまでマップに保持）
    let tx = {
        let transactions = TRANSACTIONS.lock();
        transactions
            .get(&tx_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?
            .clone()
    };

    // コミット実行（失敗した場合はトランザクションがマップに残るのでリトライ可能）
    tx.commit().map_err(|e| e.message)?;

    // 成功したのでマップから削除
    TRANSACTIONS.lock().remove(&tx_id);

    // トランザクション→プールマッピングから削除
    // 注意: 接続はCONNECTIONS/POOLED_CONNECTIONSに残し、
    // ユーザーがdb/pool-releaseで明示的に返却できるようにする
    TRANSACTION_POOLS.lock().remove(&tx_id);

    Ok(Value::Nil)
}

/// db/rollback - トランザクションをロールバック
pub fn native_rollback(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/rollback"]));
    }

    let tx_id = extract_tx_id(&args[0])?;

    // トランザクションをクローンしてからミューテックスを解放（成功するまでマップに保持）
    let tx = {
        let transactions = TRANSACTIONS.lock();
        transactions
            .get(&tx_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbTransactionNotFound, &[&tx_id]))?
            .clone()
    };

    // ロールバック実行（失敗した場合はトランザクションがマップに残るのでリトライ可能）
    tx.rollback().map_err(|e| e.message)?;

    // 成功したのでマップから削除
    TRANSACTIONS.lock().remove(&tx_id);

    // トランザクション→プールマッピングから削除
    // 注意: 接続はCONNECTIONS/POOLED_CONNECTIONSに残し、
    // ユーザーがdb/pool-releaseで明示的に返却できるようにする
    TRANSACTION_POOLS.lock().remove(&tx_id);

    Ok(Value::Nil)
}
