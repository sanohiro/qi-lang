use super::*;
use crate::builtins::db::types::*;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::i18n::{fmt_msg, MsgKey};

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
        match &args[2] {
            Value::Integer(n) if *n > 0 => *n as usize,
            Value::Integer(_) => {
                return Err(fmt_msg(
                    MsgKey::DbInvalidPoolSize,
                    &["db/create-pool", "positive integer"],
                ))
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::ThirdArgMustBe,
                    &["db/create-pool", "positive integer"],
                ))
            }
        }
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
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-acquire"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    // プールをクローンしてからミューテックスを解放（デッドロック回避）
    let pool = {
        let pools = POOLS.lock();
        pools
            .get(&pool_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?
            .clone()
    }; // pools ミューテックスはここで解放される

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
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/pool-release"]));
    }

    let pool_id = extract_pool_id(&args[0])?;
    let conn_id = extract_conn_id(&args[1])?;

    // プールをクローンしてからミューテックスを解放
    let pool = {
        let pools = POOLS.lock();
        pools
            .get(&pool_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?
            .clone()
    };

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
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-close"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    // プールを取り出してからミューテックスを解放
    let pool = {
        let mut pools = POOLS.lock();
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
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-stats"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let (available, in_use, max) = pool.stats();

    let mut map = HashMap::new();
    map.insert("available".to_string(), Value::Integer(available as i64));
    map.insert("in_use".to_string(), Value::Integer(in_use as i64));
    map.insert("max".to_string(), Value::Integer(max as i64));

    Ok(Value::Map(convert_string_map_to_mapkey(map)))
}
