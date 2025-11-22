use super::*;
use crate::builtins::db::types::*;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::i18n::{fmt_msg, MsgKey};

/// ストアドプロシージャまたはストアドファンクションを呼び出す
///
/// # 引数
/// - `conn_id` (DbConnection): データベース接続
/// - `name` (string): プロシージャ/ファンクション名
/// - `params` (vector, optional): パラメータのベクトル
///
/// # 戻り値
/// - (vector | map): プロシージャ/ファンクションの実行結果
pub fn native_call(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["db/call"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let name = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/call", "string"])),
    };

    let params = if args.len() == 3 {
        params_from_value(&args[2]).map_err(|e| e.message)?
    } else {
        vec![]
    };

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    let call_result = conn.call(name, &params).map_err(|e| e.message)?;

    // CallResultをValueに変換
    let result = match call_result {
        CallResult::Value(v) => v,
        CallResult::Rows(rows) => rows_to_value(rows),
        CallResult::Multiple(multiple) => {
            Value::Vector(multiple.into_iter().map(rows_to_value).collect())
        }
    };

    Ok(result)
}

/// db/supports? - 機能をサポートしているか確認
pub fn native_supports(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/supports?"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let feature = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/supports?", "string"],
            ))
        }
    };

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    let supported = conn.supports(feature);

    Ok(Value::Bool(supported))
}

/// db/driver-info - ドライバー情報を取得
pub fn native_driver_info(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/driver-info"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    let info = conn.driver_info().map_err(|e| e.message)?;

    // DriverInfoをマップに変換
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String(info.name));
    map.insert("version".to_string(), Value::String(info.version));
    map.insert(
        "database_version".to_string(),
        Value::String(info.database_version),
    );

    Ok(Value::Map(convert_string_map_to_mapkey(map)))
}

/// db/query-info - クエリのメタデータを取得
pub fn native_query_info(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/query-info"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/query-info", "string"],
            ))
        }
    };

    // 接続をクローンしてからミューテックスを解放
    let conn = {
        let connections = CONNECTIONS.lock();
        connections
            .get(&conn_id)
            .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
            .clone()
    };

    let info = conn.query_info(sql).map_err(|e| e.message)?;

    // QueryInfoをマップに変換
    let columns = info
        .columns
        .into_iter()
        .map(|col| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(col.name));
            map.insert("type".to_string(), Value::String(col.data_type));
            map.insert("nullable".to_string(), Value::Bool(col.nullable));
            map.insert(
                "default".to_string(),
                col.default_value.map(Value::String).unwrap_or(Value::Nil),
            );
            map.insert("primary_key".to_string(), Value::Bool(col.primary_key));
            Value::Map(convert_string_map_to_mapkey(map))
        })
        .collect();

    let mut result = HashMap::new();
    result.insert("columns".to_string(), Value::Vector(columns));
    result.insert(
        "parameter_count".to_string(),
        Value::Integer(info.parameter_count as i64),
    );

    Ok(Value::Map(convert_string_map_to_mapkey(result)))
}
