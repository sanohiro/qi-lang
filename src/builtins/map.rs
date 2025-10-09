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
