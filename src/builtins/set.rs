//! 集合演算関数

use crate::i18n::{fmt_msg, MsgKey};
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
            _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["union", "lists or vectors"])),
        }
    }

    Ok(Value::List(result))
}

/// intersect - 積集合
pub fn native_intersect(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["intersect", "2"]));
    }

    // 最初のリストをベースにする
    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["intersect", "lists or vectors"])),
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
            _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["intersect", "lists or vectors"])),
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
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["difference", "2"]));
    }

    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["difference", "lists or vectors"])),
    };

    let mut exclude = HashSet::new();
    for arg in &args[1..] {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                for item in items {
                    exclude.insert(format!("{:?}", item));
                }
            }
            _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["difference", "lists or vectors"])),
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
        return Err(fmt_msg(MsgKey::Need2Args, &["subset?"]));
    }

    let subset = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["subset?", "lists or vectors"])),
    };

    let superset = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::AllElementsMustBe, &["subset?", "lists or vectors"])),
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
