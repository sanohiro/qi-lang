use super::*;
use crate::builtins::db::types::*;
use crate::i18n::{fmt_msg, MsgKey};

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

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let conn = pool.acquire().map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/pool-release - 接続をプールに返却
pub fn native_pool_release(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/pool-release"]));
    }

    let pool_id = extract_pool_id(&args[0])?;
    let conn_id = extract_conn_id(&args[1])?;

    let pools = POOLS.lock();
    let pool = pools
        .get(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

    let mut connections = CONNECTIONS.lock();
    let conn = connections
        .remove(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    pool.release(conn);

    Ok(Value::Nil)
}

/// db/pool-close - プール全体をクローズ
pub fn native_pool_close(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/pool-close"]));
    }

    let pool_id = extract_pool_id(&args[0])?;

    let mut pools = POOLS.lock();
    let pool = pools
        .remove(&pool_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbPoolNotFound, &[&pool_id]))?;

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

    Ok(Value::Map(map.into()))
}
