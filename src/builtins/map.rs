//! マップ操作関数

use crate::i18n::{msg, MsgKey};
use crate::value::Value;

/// get - マップから値を取得
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("getには2つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::Map(m) => {
            let key = match &args[1] {
                Value::String(s) => s.clone(),
                Value::Keyword(k) => k.clone(),
                _ => return Err(msg(MsgKey::GetKeyMustBeKeyword).to_string()),
            };
            Ok(m.get(&key).cloned().unwrap_or(Value::Nil))
        }
        _ => Err(msg(MsgKey::GetFirstArgMap).to_string()),
    }
}

/// keys - マップのキーを取得
pub fn native_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("keysには1つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::Map(m) => {
            let keys: Vec<Value> = m.keys().map(|k| Value::Keyword(k.clone())).collect();
            Ok(Value::List(keys))
        }
        _ => Err(msg(MsgKey::KeysMapOnly).to_string()),
    }
}

/// vals - マップの値を取得
pub fn native_vals(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("valsには1つの引数が必要です: 実際 {}", args.len()));
    }
    match &args[0] {
        Value::Map(m) => {
            let vals: Vec<Value> = m.values().cloned().collect();
            Ok(Value::List(vals))
        }
        _ => Err(msg(MsgKey::ValsMapOnly).to_string()),
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
                    _ => return Err(msg(MsgKey::AssocKeyMustBeKeyword).to_string()),
                };
                new_map.insert(key, args[i + 1].clone());
            }
            Ok(Value::Map(new_map))
        }
        _ => Err(msg(MsgKey::AssocFirstArgMap).to_string()),
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
                    _ => return Err(msg(MsgKey::DissocKeyMustBeKeyword).to_string()),
                };
                new_map.remove(&key);
            }
            Ok(Value::Map(new_map))
        }
        _ => Err(msg(MsgKey::DissocFirstArgMap).to_string()),
    }
}
