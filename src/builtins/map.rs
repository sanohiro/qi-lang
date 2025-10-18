//! マップ操作関数
//!
//! 注: 基本的なマップ操作(get, keys, vals, assoc, dissoc, merge, get-in)は
//! core_collections.rsで実装されています。
//! このモジュールには高度なマップ操作のみを含みます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// select-keys - マップから指定したキーのみ選択
pub fn native_select_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["select-keys"]));
    }
    match (&args[0], &args[1]) {
        (Value::Map(m), Value::List(keys) | Value::Vector(keys)) => {
            let mut result = im::HashMap::new();
            for key_val in keys {
                let key = key_val.to_map_key()?;
                if let Some(v) = m.get(&key) {
                    result.insert(key, v.clone());
                }
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["select-keys", "map and list/vector"],
        )),
    }
}

/// assoc-in - ネストしたマップに値を設定
pub fn native_assoc_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["assoc-in", "3", "(map, path, value)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["assoc-in", "path"])),
    };
    let value = &args[2];

    if path.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["assoc-in", "path"]));
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            assoc_in_helper(&mut result, path, 0, value)?;
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeMap, &["assoc-in", "first argument"])),
    }
}

fn assoc_in_helper(
    map: &mut im::HashMap<String, Value>,
    path: &im::Vector<Value>,
    index: usize,
    value: &Value,
) -> Result<(), String> {
    let key = path[index].to_map_key()?;

    if index == path.len() - 1 {
        // 最後のキー：値を設定
        map.insert(key, value.clone());
    } else {
        // 中間のキー：再帰的に処理
        let next_val = map
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Value::Map(im::HashMap::new()));
        match next_val {
            Value::Map(mut inner_map) => {
                assoc_in_helper(&mut inner_map, path, index + 1, value)?;
                map.insert(key, Value::Map(inner_map));
            }
            _ => {
                // 既存の値がマップでない場合は上書き
                let mut new_map = im::HashMap::new();
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
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["dissoc-in", "2", "(map, path)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["dissoc-in", "path"])),
    };

    if path.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["dissoc-in", "path"]));
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            dissoc_in_helper(&mut result, path, 0)?;
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeMap, &["dissoc-in", "first argument"])),
    }
}

fn dissoc_in_helper(
    map: &mut im::HashMap<String, Value>,
    path: &im::Vector<Value>,
    index: usize,
) -> Result<(), String> {
    let key = path[index].to_map_key()?;

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
pub fn native_update_keys(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update-keys", "2", "(key-fn, map)"],
        ));
    }

    let key_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = im::HashMap::new();
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
        _ => Err(fmt_msg(
            MsgKey::MustBeMap,
            &["update-keys", "second argument"],
        )),
    }
}

/// update-vals - マップのすべての値に関数を適用
pub fn native_update_vals(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update-vals", "2", "(val-fn, map)"],
        ));
    }

    let val_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = im::HashMap::new();
            for (k, v) in m {
                let new_val = evaluator.apply_function(val_fn, std::slice::from_ref(v))?;
                result.insert(k.clone(), new_val);
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeMap,
            &["update-vals", "second argument"],
        )),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
///
/// 注意: update-keys, update-valsはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("map/select-keys", native_select_keys),
    ("map/assoc-in", native_assoc_in),
    ("map/dissoc-in", native_dissoc_in),
];
