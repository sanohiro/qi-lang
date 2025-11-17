use super::*;
use crate::builtins::db::traits::*;
use crate::builtins::db::types::*;
use crate::i18n::{fmt_msg, MsgKey};

#[cfg(feature = "db-mysql")]
use crate::builtins::mysql::MysqlDriver;
#[cfg(feature = "db-postgres")]
use crate::builtins::postgres::PostgresDriver;
#[cfg(feature = "db-sqlite")]
use crate::builtins::sqlite::SqliteDriver;

pub fn native_connect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["db/connect"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["db/connect", "string"])),
    };

    let opts = if args.len() == 2 {
        ConnectionOptions::from_value(&args[1]).map_err(|e| e.message)?
    } else {
        ConnectionOptions::default()
    };

    // URLからドライバーを判定
    let driver: Box<dyn DbDriver> = if url.starts_with("sqlite:") {
        #[cfg(feature = "db-sqlite")]
        {
            Box::new(SqliteDriver::new())
        }
        #[cfg(not(feature = "db-sqlite"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["sqlite (feature not enabled)"],
            ));
        }
    } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        #[cfg(feature = "db-postgres")]
        {
            Box::new(PostgresDriver::new())
        }
        #[cfg(not(feature = "db-postgres"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["postgres (feature not enabled)"],
            ));
        }
    } else if url.starts_with("mysql://") {
        #[cfg(feature = "db-mysql")]
        {
            Box::new(MysqlDriver::new())
        }
        #[cfg(not(feature = "db-mysql"))]
        {
            return Err(fmt_msg(
                MsgKey::DbUnsupportedUrl,
                &["mysql (feature not enabled)"],
            ));
        }
    } else {
        return Err(fmt_msg(MsgKey::DbUnsupportedUrl, &[url]));
    };

    let conn = driver.connect(url, &opts).map_err(|e| e.message)?;

    // 接続を保存
    let conn_id = gen_conn_id();
    CONNECTIONS.lock().insert(conn_id.clone(), conn);

    Ok(Value::String(format!("DbConnection:{}", conn_id)))
}

/// db/query - SQLクエリを実行
pub fn native_close(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/close"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let mut connections = CONNECTIONS.lock();
    let conn = connections
        .remove(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    conn.close().map_err(|e| e.message)?;

    Ok(Value::Nil)
}
