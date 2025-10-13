//! Coreコレクション操作関数
//!
//! リスト基本（28個）: first, rest, last, nth, take, drop, map, filter, reduce, pmap, cons, conj,
//!                     concat, flatten, range, reverse, zip, sort, distinct, find, every, some,
//!                     take-while, drop-while, len, count, split-at, interleave
//! マップ基本（9個）: get, keys, vals, assoc, dissoc, merge, get-in, update-in, update
//!
//! 注: map, filter, reduce, pmap, take-while, drop-while, find, every, some, update-in, update
//!     は Evaluator が必要なため、mod.rs で別途エクスポートされます

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;

// ========================================
// リスト操作（Evaluator不要）
// ========================================

/// first - リストの最初の要素
pub fn native_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["first"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(v.first().cloned().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["first", "lists or vectors"])),
    }
}

/// rest - リストの残り
pub fn native_rest(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["rest"]));
    }
    match &args[0] {
        Value::List(v) => {
            if v.is_empty() {
                Ok(Value::List(Vec::new()))
            } else {
                Ok(Value::List(v[1..].to_vec()))
            }
        }
        Value::Vector(v) => {
            if v.is_empty() {
                Ok(Value::Vector(Vec::new()))
            } else {
                Ok(Value::Vector(v[1..].to_vec()))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["rest", "lists or vectors"])),
    }
}

/// last - リストの最後の要素
pub fn native_last(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["last"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(v.last().cloned().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["last", "lists or vectors"])),
    }
}

/// nth - n番目の要素を取得
pub fn native_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["nth"]));
    }
    let index = match &args[1] {
        Value::Integer(n) => *n as usize,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["nth", "an integer"])),
    };
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(v.get(index).cloned().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["nth", "lists or vectors"])),
    }
}

/// len - 長さを取得
pub fn native_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["len"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(Value::Integer(v.len() as i64)),
        Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["len", "strings or collections"],
        )),
    }
}

/// count - 要素数を取得（lenのエイリアス）
pub fn native_count(args: &[Value]) -> Result<Value, String> {
    native_len(args)
}

/// cons - リストの先頭に要素を追加
pub fn native_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["cons"]));
    }
    match &args[1] {
        Value::Nil => Ok(Value::List(vec![args[0].clone()])),
        Value::List(v) => {
            let mut new_list = vec![args[0].clone()];
            new_list.extend(v.clone());
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = vec![args[0].clone()];
            new_vec.extend(v.clone());
            Ok(Value::List(new_vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["cons", "lists or vectors"])),
    }
}

/// conj - コレクションに要素を追加
pub fn native_conj(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["conj", "2"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut new_list = v.clone();
            for item in &args[1..] {
                new_list.insert(0, item.clone());
            }
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = v.clone();
            new_vec.extend_from_slice(&args[1..]);
            Ok(Value::Vector(new_vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["conj", "lists or vectors"])),
    }
}

/// concat - 複数のリストを連結
pub fn native_concat(args: &[Value]) -> Result<Value, String> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::List(v) | Value::Vector(v) => result.extend(v.clone()),
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["concat", "lists or vectors"])),
        }
    }
    Ok(Value::List(result))
}

/// flatten - ネストしたリストを平坦化
pub fn native_flatten(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["flatten"]));
    }
    fn flatten_value(v: &Value, result: &mut Vec<Value>) {
        match v {
            Value::List(items) | Value::Vector(items) => {
                for item in items {
                    flatten_value(item, result);
                }
            }
            _ => result.push(v.clone()),
        }
    }
    let mut result = Vec::new();
    flatten_value(&args[0], &mut result);
    Ok(Value::List(result))
}

/// range - 0からn-1までのリストを生成
pub fn native_range(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["range"]));
    }
    match &args[0] {
        Value::Integer(n) => {
            let items: Vec<Value> = (0..*n).map(Value::Integer).collect();
            Ok(Value::List(items))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["range", "integers"])),
    }
}

/// reverse - リストを反転
pub fn native_reverse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["reverse"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut reversed = v.clone();
            reversed.reverse();
            Ok(Value::List(reversed))
        }
        Value::Vector(v) => {
            let mut reversed = v.clone();
            reversed.reverse();
            Ok(Value::Vector(reversed))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["reverse", "lists or vectors"])),
    }
}

/// take - リストの最初のn要素を取得
pub fn native_take(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["take"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["take", "an integer"])),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().take(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().take(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["take", "lists or vectors"])),
    }
}

/// drop - リストの最初のn要素をスキップ
pub fn native_drop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["drop"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["drop", "an integer"])),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().skip(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().skip(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["drop", "lists or vectors"])),
    }
}

/// sort - リストをソート
pub fn native_sort(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sort"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            let mut sorted = v.clone();
            sorted.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::List(sorted))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sort", "lists or vectors"])),
    }
}

/// distinct - 重複を排除
pub fn native_distinct(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["distinct"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            let mut result = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for item in v {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["distinct", "lists or vectors"])),
    }
}

/// zip - 2つのリストを組み合わせる
pub fn native_zip(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["zip"]));
    }
    match (&args[0], &args[1]) {
        (Value::List(a), Value::List(b)) | (Value::Vector(a), Value::Vector(b)) => {
            let result: Vec<Value> = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Vector(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["zip", "lists or vectors"])),
    }
}

// ========================================
// マップ操作（Evaluator不要）
// ========================================

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
            // 削除するキーの数を計算して capacity を確保
            let num_remove_keys = args.len() - 1;
            let new_capacity = m.len().saturating_sub(num_remove_keys);
            let mut new_map = HashMap::with_capacity(new_capacity.max(m.len() / 2));

            // キーのセットを作成
            let mut keys_to_remove = std::collections::HashSet::with_capacity(num_remove_keys);
            for arg in &args[1..] {
                let key = match arg {
                    Value::String(s) => s.clone(),
                    Value::Keyword(k) => k.clone(),
                    _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
                };
                keys_to_remove.insert(key);
            }

            // 削除するキー以外をコピー
            new_map.extend(
                m.iter()
                    .filter(|(k, _)| !keys_to_remove.contains(*k))
                    .map(|(k, v)| (k.clone(), v.clone())),
            );

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
    let mut result = HashMap::new();
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

/// get-in - ネストしたマップから値を取得
pub fn native_get_in(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["get-in", "2 or 3", "(map, path, default?)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["get-in", "path"])),
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
            _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
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
