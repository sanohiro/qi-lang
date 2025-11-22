use super::*;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::i18n::{fmt_msg, MsgKey};

/// データベース内のテーブル一覧を取得する
///
/// # 引数
/// - `conn_id` (DbConnection): データベース接続
///
/// # 戻り値
/// - (vector): テーブル名の文字列ベクトル
///
/// # 例
/// ```qi
/// (let tables (db/tables conn))
/// ;; => ["users" "posts" "comments"]
/// ```
pub fn native_tables(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["db/tables"]));
    }

    let conn_id = extract_conn_id(&args[0])?;

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let tables = conn.tables().map_err(|e| e.message)?;

    Ok(Value::Vector(
        tables.into_iter().map(Value::String).collect(),
    ))
}

/// db/columns - テーブルのカラム情報を取得
pub fn native_columns(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/columns"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/columns", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let columns = conn.columns(table).map_err(|e| e.message)?;

    // Vec<ColumnInfo>をValue::Vectorに変換
    let result = columns
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

    Ok(Value::Vector(result))
}

/// db/indexes - テーブルのインデックス一覧を取得
pub fn native_indexes(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/indexes"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["db/indexes", "string"])),
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let indexes = conn.indexes(table).map_err(|e| e.message)?;

    // Vec<IndexInfo>をValue::Vectorに変換
    let result = indexes
        .into_iter()
        .map(|idx| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(idx.name));
            map.insert("table".to_string(), Value::String(idx.table));
            map.insert(
                "columns".to_string(),
                Value::Vector(idx.columns.into_iter().map(Value::String).collect()),
            );
            map.insert("unique".to_string(), Value::Bool(idx.unique));
            Value::Map(convert_string_map_to_mapkey(map))
        })
        .collect();

    Ok(Value::Vector(result))
}

/// db/foreign-keys - テーブルの外部キー一覧を取得
pub fn native_foreign_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["db/foreign-keys"]));
    }

    let conn_id = extract_conn_id(&args[0])?;
    let table = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["db/foreign-keys", "string"],
            ))
        }
    };

    let connections = CONNECTIONS.lock();
    let conn = connections
        .get(&conn_id)
        .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?;

    let foreign_keys = conn.foreign_keys(table).map_err(|e| e.message)?;

    // Vec<ForeignKeyInfo>をValue::Vectorに変換
    let result = foreign_keys
        .into_iter()
        .map(|fk| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String(fk.name));
            map.insert("table".to_string(), Value::String(fk.table));
            map.insert(
                "columns".to_string(),
                Value::Vector(fk.columns.into_iter().map(Value::String).collect()),
            );
            map.insert(
                "referenced_table".to_string(),
                Value::String(fk.referenced_table),
            );
            map.insert(
                "referenced_columns".to_string(),
                Value::Vector(
                    fk.referenced_columns
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
            Value::Map(convert_string_map_to_mapkey(map))
        })
        .collect();

    Ok(Value::Vector(result))
}
