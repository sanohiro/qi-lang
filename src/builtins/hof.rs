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

/// update - マップの値を関数で更新
pub fn native_update(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("update requires 3 arguments (map, key, fn)".to_string());
    }

    let map = &args[0];
    let key_val = &args[1];
    let func = &args[2];

    match map {
        Value::Map(m) => {
            let key = match key_val {
                Value::String(s) => s.clone(),
                Value::Keyword(k) => k.clone(),
                _ => return Err("update: key must be string or keyword".to_string()),
            };

            let current_value = m.get(&key).cloned().unwrap_or(Value::Nil);
            let new_value = evaluator.apply_function(func, &[current_value])?;

            let mut result = m.clone();
            result.insert(key, new_value);
            Ok(Value::Map(result))
        }
        _ => Err("update: first argument must be a map".to_string()),
    }
}

/// update-in - ネストしたマップの値を関数で更新
pub fn native_update_in(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("update-in requires 3 arguments (map, path, fn)".to_string());
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err("update-in: path must be a list or vector".to_string()),
    };
    let func = &args[2];

    if path.is_empty() {
        return Err("update-in: path cannot be empty".to_string());
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            update_in_helper(&mut result, path, 0, func, evaluator)?;
            Ok(Value::Map(result))
        }
        _ => Err("update-in: first argument must be a map".to_string()),
    }
}

fn update_in_helper(
    map: &mut std::collections::HashMap<String, Value>,
    path: &[Value],
    index: usize,
    func: &Value,
    evaluator: &mut Evaluator,
) -> Result<(), String> {
    let key = match &path[index] {
        Value::String(s) => s.clone(),
        Value::Keyword(k) => k.clone(),
        _ => return Err("update-in: keys must be strings or keywords".to_string()),
    };

    if index == path.len() - 1 {
        // 最後のキー：値を更新
        let current_value = map.get(&key).cloned().unwrap_or(Value::Nil);
        let new_value = evaluator.apply_function(func, &[current_value])?;
        map.insert(key, new_value);
    } else {
        // 中間のキー：再帰的に処理
        let next_val = map.get(&key).cloned().unwrap_or_else(|| Value::Map(std::collections::HashMap::new()));
        match next_val {
            Value::Map(mut inner_map) => {
                update_in_helper(&mut inner_map, path, index + 1, func, evaluator)?;
                map.insert(key, Value::Map(inner_map));
            }
            _ => {
                // 既存の値がマップでない場合は上書き
                let mut new_map = std::collections::HashMap::new();
                update_in_helper(&mut new_map, path, index + 1, func, evaluator)?;
                map.insert(key, Value::Map(new_map));
            }
        }
    }
    Ok(())
}

/// identity - 引数をそのまま返す
pub fn native_identity(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("identity requires 1 argument".to_string());
    }
    Ok(args[0].clone())
}

/// constantly - 常に同じ値を返す関数を生成
pub fn native_constantly(args: &[Value]) -> Result<Value, String> {
    use std::rc::Rc;

    if args.len() != 1 {
        return Err("constantly requires 1 argument".to_string());
    }
    let value = args[0].clone();
    // 単純に値を返すだけの関数を作る（評価時に特別処理）
    Ok(Value::Function(Rc::new(crate::value::Function {
        params: vec!["_".to_string()],
        body: crate::value::Expr::Symbol("__constantly_value__".to_string()),
        env: {
            let mut env = crate::value::Env::new();
            env.set("__constantly_value__".to_string(), value);
            env
        },
        is_variadic: false,
    })))
}

/// comp - 関数合成（右から左に適用）
pub fn native_comp(args: &[Value], _evaluator: &mut Evaluator) -> Result<Value, String> {
    use std::rc::Rc;

    if args.is_empty() {
        return Err("comp requires at least 1 function".to_string());
    }

    // 1つの関数の場合はそのまま返す
    if args.len() == 1 {
        return Ok(args[0].clone());
    }

    // 複数の関数の場合は合成された関数を返す
    let funcs = args.to_vec();
    Ok(Value::Function(Rc::new(crate::value::Function {
        params: vec!["x".to_string()],
        body: crate::value::Expr::Symbol("__comp_placeholder__".to_string()),
        env: {
            let mut env = crate::value::Env::new();
            env.set("__comp_funcs__".to_string(), Value::List(funcs));
            env
        },
        is_variadic: false,
    })))
}

/// partial - 部分適用
pub fn native_partial(args: &[Value]) -> Result<Value, String> {
    use std::rc::Rc;

    if args.len() < 2 {
        return Err("partial requires at least 2 arguments (function and at least 1 arg)".to_string());
    }

    let func = args[0].clone();
    let partial_args: Vec<Value> = args[1..].to_vec();

    Ok(Value::Function(Rc::new(crate::value::Function {
        params: vec!["&rest".to_string()],
        body: crate::value::Expr::Symbol("__partial_placeholder__".to_string()),
        env: {
            let mut env = crate::value::Env::new();
            env.set("__partial_func__".to_string(), func);
            env.set("__partial_args__".to_string(), Value::List(partial_args));
            env
        },
        is_variadic: true,
    })))
}

/// apply - リストを引数として関数適用（既存のものを公開用に再実装）
pub fn native_apply_public(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("apply requires 2 arguments (function and list)".to_string());
    }

    let func = &args[0];
    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            evaluator.apply_function(func, items)
        }
        _ => Err("apply: second argument must be a list or vector".to_string()),
    }
}

/// count-by - 述語でカウント
pub fn native_count_by(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    use std::collections::HashMap;

    if args.len() != 2 {
        return Err("count-by requires 2 arguments (predicate, collection)".to_string());
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut counts: HashMap<String, i64> = HashMap::new();
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                let key = if result.is_truthy() { "true" } else { "false" };
                *counts.entry(key.to_string()).or_insert(0) += 1;
            }

            let mut result = HashMap::new();
            for (key, count) in counts {
                result.insert(key, Value::Integer(count));
            }
            Ok(Value::Map(result))
        }
        _ => Err("count-by: second argument must be a list or vector".to_string()),
    }
}
