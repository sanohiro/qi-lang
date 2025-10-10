//! マップ操作関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

/// get - マップから値を取得
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["get"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let key = match &args[1] {
                Value::String(s) => s.clone(),
                Value::Keyword(k) => k.clone(),
                _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
            };
            Ok(m.get(&key).cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["get", "a map"])),
    }
}

/// keys - マップのキーを取得
pub fn native_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["keys"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let keys: Vec<Value> = m.keys().map(|k| Value::Keyword(k.clone())).collect();
            Ok(Value::List(keys))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["keys", "maps"])),
    }
}

/// vals - マップの値を取得
pub fn native_vals(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["vals"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let vals: Vec<Value> = m.values().cloned().collect();
            Ok(Value::List(vals))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["vals", "maps"])),
    }
}

/// assoc - マップに新しいキー・値のペアを追加
pub fn native_assoc(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        return Err(msg(MsgKey::AssocMapAndKeyValues).to_string());
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_map = m.clone();
            for i in (1..args.len()).step_by(2) {
                let key = match &args[i] {
                    Value::String(s) => s.clone(),
                    Value::Keyword(k) => k.clone(),
                    _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
                };
                new_map.insert(key, args[i + 1].clone());
            }
            Ok(Value::Map(new_map))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["assoc", "a map"])),
    }
}

/// dissoc - マップから指定したキーを削除
pub fn native_dissoc(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(msg(MsgKey::DissocMapAndKeys).to_string());
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_map = m.clone();
            for arg in &args[1..] {
                let key = match arg {
                    Value::String(s) => s.clone(),
                    Value::Keyword(k) => k.clone(),
                    _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
                };
                new_map.remove(&key);
            }
            Ok(Value::Map(new_map))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["dissoc", "a map"])),
    }
}

/// merge - 複数のマップをマージ
pub fn native_merge(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["merge", "1"]));
    }
    let mut result = std::collections::HashMap::new();
    for arg in args {
        match arg {
            Value::Map(m) => {
                for (k, v) in m {
                    result.insert(k.clone(), v.clone());
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["merge", "maps"])),
        }
    }
    Ok(Value::Map(result))
}

/// select-keys - マップから指定したキーのみ選択
pub fn native_select_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["select-keys"]));
    }
    match (&args[0], &args[1]) {
        (Value::Map(m), Value::List(keys) | Value::Vector(keys)) => {
            let mut result = std::collections::HashMap::new();
            for key_val in keys {
                let key = match key_val {
                    Value::String(s) => s.clone(),
                    Value::Keyword(k) => k.clone(),
                    _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
                };
                if let Some(v) = m.get(&key) {
                    result.insert(key, v.clone());
                }
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["select-keys", "map and list/vector"])),
    }
}

/// get-in - ネストしたマップから値を取得
pub fn native_get_in(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err("get-in requires 2 or 3 arguments (map, path, default?)".to_string());
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err("get-in: path must be a list or vector".to_string()),
    };
    let default = if args.len() == 3 {
        args[2].clone()
    } else {
        Value::Nil
    };

    let mut current = map.clone();
    for key_val in path {
        let key = match key_val {
            Value::String(s) => s.clone(),
            Value::Keyword(k) => k.clone(),
            _ => return Err("get-in: keys must be strings or keywords".to_string()),
        };

        match current {
            Value::Map(m) => {
                current = m.get(&key).cloned().unwrap_or(Value::Nil);
                if matches!(current, Value::Nil) {
                    return Ok(default);
                }
            }
            _ => return Ok(default),
        }
    }

    Ok(current)
}

/// assoc-in - ネストしたマップに値を設定
pub fn native_assoc_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("assoc-in requires 3 arguments (map, path, value)".to_string());
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err("assoc-in: path must be a list or vector".to_string()),
    };
    let value = &args[2];

    if path.is_empty() {
        return Err("assoc-in: path cannot be empty".to_string());
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            assoc_in_helper(&mut result, path, 0, value)?;
            Ok(Value::Map(result))
        }
        _ => Err("assoc-in: first argument must be a map".to_string()),
    }
}

fn assoc_in_helper(
    map: &mut std::collections::HashMap<String, Value>,
    path: &[Value],
    index: usize,
    value: &Value,
) -> Result<(), String> {
    let key = match &path[index] {
        Value::String(s) => s.clone(),
        Value::Keyword(k) => k.clone(),
        _ => return Err("assoc-in: keys must be strings or keywords".to_string()),
    };

    if index == path.len() - 1 {
        // 最後のキー：値を設定
        map.insert(key, value.clone());
    } else {
        // 中間のキー：再帰的に処理
        let next_val = map.get(&key).cloned().unwrap_or_else(|| Value::Map(std::collections::HashMap::new()));
        match next_val {
            Value::Map(mut inner_map) => {
                assoc_in_helper(&mut inner_map, path, index + 1, value)?;
                map.insert(key, Value::Map(inner_map));
            }
            _ => {
                // 既存の値がマップでない場合は上書き
                let mut new_map = std::collections::HashMap::new();
                assoc_in_helper(&mut new_map, path, index + 1, value)?;
                map.insert(key, Value::Map(new_map));
            }
        }
    }
    Ok(())
}

/// dissoc-in - ネストしたマップからキーを削除
pub fn native_dissoc_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("dissoc-in requires 2 arguments (map, path)".to_string());
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err("dissoc-in: path must be a list or vector".to_string()),
    };

    if path.is_empty() {
        return Err("dissoc-in: path cannot be empty".to_string());
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            dissoc_in_helper(&mut result, path, 0)?;
            Ok(Value::Map(result))
        }
        _ => Err("dissoc-in: first argument must be a map".to_string()),
    }
}

fn dissoc_in_helper(
    map: &mut std::collections::HashMap<String, Value>,
    path: &[Value],
    index: usize,
) -> Result<(), String> {
    let key = match &path[index] {
        Value::String(s) => s.clone(),
        Value::Keyword(k) => k.clone(),
        _ => return Err("dissoc-in: keys must be strings or keywords".to_string()),
    };

    if index == path.len() - 1 {
        // 最後のキー：削除
        map.remove(&key);
    } else {
        // 中間のキー：再帰的に処理
        if let Some(Value::Map(inner_map)) = map.get_mut(&key) {
            let mut inner_clone = inner_map.clone();
            dissoc_in_helper(&mut inner_clone, path, index + 1)?;
            map.insert(key, Value::Map(inner_clone));
        }
    }
    Ok(())
}

/// update-keys - マップのすべてのキーに関数を適用
pub fn native_update_keys(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("update-keys requires 2 arguments (key-fn, map)".to_string());
    }

    let key_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = std::collections::HashMap::new();
            for (k, v) in m {
                let key_val = Value::String(k.clone());
                let new_key_val = evaluator.apply_function(key_fn, &[key_val])?;
                let new_key = match new_key_val {
                    Value::String(s) => s,
                    Value::Keyword(k) => k,
                    _ => format!("{:?}", new_key_val),
                };
                result.insert(new_key, v.clone());
            }
            Ok(Value::Map(result))
        }
        _ => Err("update-keys: second argument must be a map".to_string()),
    }
}

/// update-vals - マップのすべての値に関数を適用
pub fn native_update_vals(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("update-vals requires 2 arguments (val-fn, map)".to_string());
    }

    let val_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = std::collections::HashMap::new();
            for (k, v) in m {
                let new_val = evaluator.apply_function(val_fn, &[v.clone()])?;
                result.insert(k.clone(), new_val);
            }
            Ok(Value::Map(result))
        }
        _ => Err("update-vals: second argument must be a map".to_string()),
    }
}
