use super::*;
use crate::check_args;
use crate::builtins::db::types::*;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::builtins::value_helpers::to_positive_usize;
use crate::i18n::{fmt_msg, MsgKey};
use crate::with_global;

/// データベース接続プールを作成する
///
/// # 引数
/// - `url` (string): 接続URL
/// - `options` (map, optional): 接続オプション
/// - `max-connections` (integer, optional): プール内の最大接続数 (デフォルト: 10)
///
/// # 戻り値
/// - (DbPool): データベース接続プール
///
/// # 例
/// ```qi
/// (let pool (db/create-pool "postgresql://localhost/mydb" {} 20))
/// ```
pub fn native_create_pool(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 3 {
        return Err(fmt_msg(
            MsgKey::DbNeed1To3Args,
            &["db/create-pool", &args.len().to_string()],
        ));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["db/create-pool", "string"],
            ))
        }
    };

    let opts = if args.len() >= 2 {
        ConnectionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        ConnectionOptions::default()
    };

    let max_connections = if args.len() >= 3 {
        to_positive_usize(&args[2], "db/create-pool", "max-connections")?
    } else {
        10 // デフォルトは10接続
    };

    // プールを作成
    let pool = DbPool::new(url, opts, max_connections);
    let pool_id = gen_pool_id();
    POOLS.lock().insert(pool_id.clone(), pool);

    Ok(Value::String(format!("DbPool:{}", pool_id)))
}

/// db/pool-acquire - プールから接続を取得
pub fn native_pool_acquire(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "db/pool-acquire");

    let pool_id = extract_pool_id(&args[0])?;

    // デッドロック回避: プールをクローンしてからミューテックスを解放
    let pool = with_global!(POOLS, &pool_id, MsgKey::DbPoolNotFound);

    // ブロッキング操作をミューテックス外で実行
    let conn = pool.acquire().map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    // プール接続として追跡（db/closeで誤って閉じられないようにする）
    POOLED_CONNECTIONS.lock().insert(conn_id.clone(), pool_id);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/pool-release - 接続をプールに返却
pub fn native_pool_release(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "db/pool-release");

    let pool_id = extract_pool_id(&args[0])?;
    let conn_id = extract_conn_id(&args[1])?;

    // 接続が指定されたプールから取得されたものか確認
    {
        let pooled_conns = POOLED_CONNECTIONS.lock();
        match pooled_conns.get(&conn_id) {
            Some(actual_pool_id) if actual_pool_id == &pool_id => {
                // OK: 正しいプールに返却しようとしている
            }
            Some(actual_pool_id) => {
                return Err(format!(
                    "Connection {} belongs to pool {}, not {}",
                    conn_id, actual_pool_id, pool_id
                ));
            }
            None => {
                return Err(format!("Connection {} is not a pooled connection", conn_id));
            }
        }
    }

    // 接続がアクティブなトランザクションに参加していないか確認（CRITICAL）
    // トランザクション中の接続をプールに戻すと、同じ接続が複数のスレッドで共有される
    {
        let tx_pools = TRANSACTION_POOLS.lock();
        for (tx_id, (_, tx_conn_id)) in tx_pools.iter() {
            if tx_conn_id == &conn_id {
                return Err(format!(
                    "Connection {} has an active transaction {}. Commit or rollback before returning to pool.",
                    conn_id, tx_id
                ));
            }
        }
    }

    let pool = with_global!(POOLS, &pool_id, MsgKey::DbPoolNotFound);

    let conn = {
        let mut connections = CONNECTIONS.lock();
        connections
            .remove(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
    };

    // プール接続の追跡から削除
    POOLED_CONNECTIONS.lock().remove(&conn_id);

    pool.release(conn);

    Ok(Value::Nil)
}

/// db/pool-close - プール全体をクローズ
pub fn native_pool_close(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "db/pool-close");

    let pool_id = extract_pool_id(&args[0])?;

    // CRITICAL: POOLED_CONNECTIONSとPOOLSを同時にロックして競合状態を防ぐ
    // 1. outstanding接続のチェック
    // 2. プールの削除
    // この2つの操作の間にdb/pool-acquireが割り込むとデッドロックするため、
    // 両方のロックを同時に保持する必要がある
    let pool = {
        let pooled_conns = POOLED_CONNECTIONS.lock();
        let mut pools = POOLS.lock();

        // チェックアウト中の接続がないか確認
        let outstanding: Vec<String> = pooled_conns
            .iter()
            .filter(|(_, pid)| *pid == &pool_id)
            .map(|(cid, _)| cid.clone())
            .collect();

        if !outstanding.is_empty() {
            return Err(format!(
                "Pool {} has {} connection(s) still checked out: {}. Release all connections before closing pool.",
                pool_id,
                outstanding.len(),
                outstanding.join(", ")
            ));
        }

        // プールを削除（pooled_connsのロックを保持したまま）
        pools
            .remove(&pool_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?
    };

    // ブロッキング操作をミューテックス外で実行
    pool.close().map_err(|e| e.message)?;

    Ok(Value::Nil)
}

/// db/pool-stats - プールの統計情報を取得
pub fn native_pool_stats(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "db/pool-stats");

    let pool_id = extract_pool_id(&args[0])?;

    // ロック保持時間を最小化: プールをクローンしてからミューテックスを解放
    let pool = with_global!(POOLS, &pool_id, MsgKey::DbPoolNotFound);

    let (available, in_use, max) = pool.stats();

    let mut map = HashMap::new();
    map.insert("available".to_string(), Value::Integer(available as i64));
    map.insert("in_use".to_string(), Value::Integer(in_use as i64));
    map.insert("max".to_string(), Value::Integer(max as i64));

    Ok(Value::Map(convert_string_map_to_mapkey(map)))
}
