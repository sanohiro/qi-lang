use super::*;
use crate::builtins::db::types::*;

impl From<DbError> for Value {
    fn from(err: DbError) -> Self {
        Value::String(format!("DbError: {}", err.message))
    }
}

/// 行をQi ValueのMapに変換
pub fn row_to_value(row: Row) -> Value {
    Value::Map(row)
}

/// 複数行をQi ValueのVectorに変換
pub fn rows_to_value(rows: Rows) -> Value {
    Value::Vector(rows.into_iter().map(row_to_value).collect())
}

/// Qi Valueからパラメータを抽出
pub fn params_from_value(params: &Value) -> DbResult<Vec<Value>> {
    match params {
        Value::Vector(vec) => Ok(vec.iter().cloned().collect()),
        Value::Nil => Ok(vec![]),
        _ => Err(DbError::new("Parameters must be a vector")),
    }
}
