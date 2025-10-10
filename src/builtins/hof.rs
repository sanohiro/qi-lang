//! 高階関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// map - リストの各要素に関数を適用
pub fn native_map(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["map"]));
    }

    let func = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut results = Vec::new();
            for item in items {
                let result = evaluator.apply_function(func, std::slice::from_ref(item))?;
                results.push(result);
            }
            Ok(Value::List(results))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["map (2nd arg)", "lists or vectors"])),
    }
}

/// filter - リストから条件を満たす要素を抽出
pub fn native_filter(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["filter"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut results = Vec::new();
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    results.push(item.clone());
                }
            }
            Ok(Value::List(results))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["filter (2nd arg)", "lists or vectors"])),
    }
}

/// reduce - リストを畳み込み
pub fn native_reduce(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 && args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["reduce"]));
    }

    let func = &args[0];
    let collection = &args[1];
    let init = if args.len() == 3 {
        Some(args[2].clone())
    } else {
        None
    };

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(init.unwrap_or(Value::Nil));
            }

            let (start_idx, mut acc) = if let Some(initial) = init {
                (0, initial)
            } else {
                (1, items[0].clone())
            };

            for item in &items[start_idx..] {
                acc = evaluator.apply_function(func, &[acc, item.clone()])?;
            }
            Ok(acc)
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["reduce (2nd arg)", "lists or vectors"])),
    }
}

/// pmap - 並列map（現在はシングルスレッド実装、将来の並列化に備えて）
///
/// 注: 現在の実装はmapと同じ動作をします。
/// 将来、Evaluatorをスレッドセーフにする際に並列化を実装予定。
pub fn native_pmap(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    // 現在はmapと同じ実装
    native_map(args, evaluator)
}

/// apply - 関数にリストを引数として適用（未使用だが将来のため残す）
#[allow(dead_code)]
pub fn native_apply(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["apply"]));
    }

    let func = &args[0];
    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            evaluator.apply_function(func, items)
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["apply (2nd arg)", "lists or vectors"])),
    }
}

/// partition - 述語でリストを2つに分割
pub fn native_partition(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["partition"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut truthy = Vec::new();
            let mut falsy = Vec::new();
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    truthy.push(item.clone());
                } else {
                    falsy.push(item.clone());
                }
            }
            Ok(Value::Vector(vec![
                Value::List(truthy),
                Value::List(falsy),
            ]))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["partition (2nd arg)", "lists or vectors"])),
    }
}

/// group-by - キー関数でリストをグループ化
pub fn native_group_by(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    use std::collections::HashMap;

    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["group-by"]));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
            for item in items {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;
                let key_str = format!("{:?}", key);
                groups.entry(key_str).or_insert_with(Vec::new).push(item.clone());
            }

            let mut result = HashMap::new();
            for (key_str, values) in groups {
                result.insert(key_str, Value::List(values));
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["group-by (2nd arg)", "lists or vectors"])),
    }
}

/// map-lines - 文字列の各行に関数を適用
pub fn native_map_lines(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["map-lines"]));
    }

    let func = &args[0];
    let text = &args[1];

    match text {
        Value::String(s) => {
            let lines: Vec<&str> = s.lines().collect();
            let mut results = Vec::new();
            for line in lines {
                let result = evaluator.apply_function(func, &[Value::String(line.to_string())])?;
                if let Value::String(transformed) = result {
                    results.push(transformed);
                } else {
                    return Err("map-lines: function must return string".to_string());
                }
            }
            Ok(Value::String(results.join("\n")))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["map-lines (2nd arg)", "strings"])),
    }
}
