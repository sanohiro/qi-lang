//! 集合演算関数

use crate::value::Value;
use std::collections::HashSet;

/// union - 和集合
pub fn native_union(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(vec![]));
    }

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                for item in items {
                    let key = format!("{:?}", item);
                    if seen.insert(key) {
                        result.push(item.clone());
                    }
                }
            }
            _ => return Err("union: all arguments must be lists or vectors".to_string()),
        }
    }

    Ok(Value::List(result))
}

/// intersect - 積集合
pub fn native_intersect(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("intersect requires at least 2 arguments".to_string());
    }

    // 最初のリストをベースにする
    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err("intersect: all arguments must be lists or vectors".to_string()),
    };

    let mut result: HashSet<String> = first.iter()
        .map(|v| format!("{:?}", v))
        .collect();

    // 他のリストとの積集合を取る
    for arg in &args[1..] {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                let set: HashSet<String> = items.iter()
                    .map(|v| format!("{:?}", v))
                    .collect();
                result.retain(|k| set.contains(k));
            }
            _ => return Err("intersect: all arguments must be lists or vectors".to_string()),
        }
    }

    // 元の値を復元
    let values: Vec<Value> = first.iter()
        .filter(|v| result.contains(&format!("{:?}", v)))
        .cloned()
        .collect();

    Ok(Value::List(values))
}

/// difference - 差集合（第1引数から第2引数以降を除く）
pub fn native_difference(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("difference requires at least 2 arguments".to_string());
    }

    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err("difference: all arguments must be lists or vectors".to_string()),
    };

    let mut exclude = HashSet::new();
    for arg in &args[1..] {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                for item in items {
                    exclude.insert(format!("{:?}", item));
                }
            }
            _ => return Err("difference: all arguments must be lists or vectors".to_string()),
        }
    }

    let values: Vec<Value> = first.iter()
        .filter(|v| !exclude.contains(&format!("{:?}", v)))
        .cloned()
        .collect();

    Ok(Value::List(values))
}

/// subset? - 部分集合判定
pub fn native_subset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("subset? requires 2 arguments".to_string());
    }

    let subset = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err("subset?: arguments must be lists or vectors".to_string()),
    };

    let superset = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err("subset?: arguments must be lists or vectors".to_string()),
    };

    let superset_keys: HashSet<String> = superset.iter()
        .map(|v| format!("{:?}", v))
        .collect();

    for item in subset {
        if !superset_keys.contains(&format!("{:?}", item)) {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}
