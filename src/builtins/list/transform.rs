//! リスト操作 - 変換関数

use super::helpers::values_equal;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// split-at - 指定位置でリストを分割
pub fn native_split_at(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["split-at", "2", "(index, collection)"],
        ));
    }

    let index = match &args[0] {
        Value::Integer(n) => *n as usize,
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["split-at", "index"])),
    };

    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            let split_point = index.min(items.len());

            // im::Vector::split_atを使用（効率的な分割）
            let (left, right) = items.clone().split_at(split_point);

            Ok(Value::Vector(
                vec![Value::List(left), Value::List(right)].into(),
            ))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["split-at (2nd arg)", "second argument"],
        )),
    }
}

/// interleave - 2つのリストを交互に組み合わせる
pub fn native_interleave(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["interleave"]));
    }

    let list1 = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["interleave (1st arg)", "first argument"],
            ))
        }
    };

    let list2 = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["interleave (2nd arg)", "second argument"],
            ))
        }
    };

    // im::Vectorを直接使用（中間Vec排除）
    let mut result = im::Vector::new();
    let min_len = list1.len().min(list2.len());
    for i in 0..min_len {
        result.push_back(list1[i].clone());
        result.push_back(list2[i].clone());
    }

    Ok(Value::List(result))
}

/// frequencies - 要素の出現回数をカウント
pub fn native_frequencies(args: &[Value]) -> Result<Value, String> {
    use std::collections::HashMap;

    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["frequencies"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let mut counts: HashMap<String, i64> = HashMap::new();
            for item in items {
                let key = format!("{:?}", item);
                *counts.entry(key).or_insert(0) += 1;
            }

            let mut result = crate::new_hashmap();
            for (key, count) in counts {
                result.insert(crate::value::MapKey::String(key), Value::Integer(count));
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["frequencies", "argument"],
        )),
    }
}

/// chunk - 固定サイズでリストを分割
pub fn native_chunk(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["chunk", "2", "(size, collection)"],
        ));
    }

    let size = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err(fmt_msg(MsgKey::MustBePositive, &["chunk", "size"])),
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["chunk", "size"])),
    };

    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            // im::Vector直接使用（中間Vec排除）
            let mut result = im::Vector::new();
            let mut current_chunk = im::Vector::new();

            for item in items {
                current_chunk.push_back(item.clone());
                if current_chunk.len() == size {
                    result.push_back(Value::List(current_chunk));
                    current_chunk = im::Vector::new();
                }
            }

            // 残りの要素を追加
            if !current_chunk.is_empty() {
                result.push_back(Value::List(current_chunk));
            }

            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["chunk (2nd arg)", "second argument"],
        )),
    }
}

/// zipmap - 2つのコレクションからマップを作成
pub fn native_zipmap(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["zipmap", "2", "(keys, vals)"],
        ));
    }

    let keys = match &args[0] {
        Value::List(v) | Value::Vector(v) => v,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["zipmap (1st arg)", "first argument"],
            ))
        }
    };

    let vals = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["zipmap (2nd arg)", "second argument"],
            ))
        }
    };

    let mut result = crate::new_hashmap();
    for (key, val) in keys.iter().zip(vals.iter()) {
        let map_key = key.to_map_key().unwrap_or_else(|_| {
            crate::value::MapKey::String(format!("{:?}", key))
        });
        result.insert(map_key, val.clone());
    }

    Ok(Value::Map(result))
}

/// take-nth - n番目ごとの要素を取得
pub fn native_take_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["take-nth", "2", "(n, collection)"],
        ));
    }

    let n = match &args[0] {
        Value::Integer(i) => {
            if *i <= 0 {
                return Err(fmt_msg(MsgKey::MustBePositive, &["take-nth", "n"]));
            }
            *i as usize
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeInteger,
                &["take-nth", "first argument"],
            ))
        }
    };

    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["take-nth (2nd arg)", "second argument"],
            ))
        }
    };

    // im::Vector直接構築（中間Vec排除）
    let result: im::Vector<Value> = collection
        .iter()
        .enumerate()
        .filter(|(i, _)| i % n == 0)
        .map(|(_, v)| v.clone())
        .collect();

    Ok(Value::List(result))
}

/// dedupe - 連続する重複を除去
pub fn native_dedupe(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["dedupe"]));
    }

    let collection = match &args[0] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["dedupe", "argument"])),
    };

    if collection.is_empty() {
        return Ok(Value::List(im::Vector::new()));
    }

    // im::Vector直接構築（中間Vec排除）
    let mut result = im::Vector::new();
    let mut last: Option<&Value> = None;

    for item in collection {
        if let Some(prev) = last {
            if !values_equal(prev, item) {
                result.push_back(item.clone());
            }
        } else {
            result.push_back(item.clone());
        }
        last = Some(item);
    }

    Ok(Value::List(result))
}

/// drop-last - 最後のn要素を削除
pub fn native_drop_last(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["drop-last", "2", "(n, collection)"],
        ));
    }

    let n = match &args[0] {
        Value::Integer(i) => {
            if *i < 0 {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["drop-last", "n"]));
            }
            *i as usize
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeInteger,
                &["drop-last", "first argument"],
            ))
        }
    };

    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["drop-last (2nd arg)", "second argument"],
            ))
        }
    };

    let take_count = if collection.len() > n {
        collection.len() - n
    } else {
        0
    };

    // im::Vector直接構築（中間Vec排除）
    let result: im::Vector<Value> = collection.iter().take(take_count).cloned().collect();
    Ok(Value::List(result))
}
