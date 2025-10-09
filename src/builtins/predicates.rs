//! 述語関数（型判定など）

use crate::value::Value;

/// empty? - コレクションが空かどうか判定
pub fn native_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("empty?には1つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::Nil => Ok(Value::Bool(true)),
        Value::List(v) | Value::Vector(v) => Ok(Value::Bool(v.is_empty())),
        Value::Map(m) => Ok(Value::Bool(m.is_empty())),
        Value::String(s) => Ok(Value::Bool(s.is_empty())),
        _ => Err("empty?はコレクションまたは文字列のみ受け付けます".to_string()),
    }
}

/// nil? - nilかどうか判定
pub fn native_nil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("nil?には1つの引数が必要です".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

/// list? - リストかどうか判定
pub fn native_list_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("list?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

/// vector? - ベクタかどうか判定
pub fn native_vector_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("vector?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Vector(_))))
}

/// map? - マップかどうか判定
pub fn native_map_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("map?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Map(_))))
}

/// string? - 文字列かどうか判定
pub fn native_string_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("string?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// integer? - 整数かどうか判定
pub fn native_integer_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("integer?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Integer(_))))
}

/// float? - 浮動小数点数かどうか判定
pub fn native_float_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("float?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Float(_))))
}

/// keyword? - キーワードかどうか判定
pub fn native_keyword_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("keyword?には1つの引数が必要です: 実際 {}", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Keyword(_))))
}
