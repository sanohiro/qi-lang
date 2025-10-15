//! データ構造 - Queue, Stack等

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;

/// queue/new - 空のキューを作成
pub fn native_queue_new(_args: &[Value]) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert("type".to_string(), Value::Keyword("queue".to_string()));
    map.insert("items".to_string(), Value::List(Vec::new()));
    Ok(Value::Map(map))
}

/// queue/enqueue - 要素をキューに追加（末尾）
pub fn native_queue_enqueue(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["queue/enqueue"]));
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeQueue,
                &["queue/enqueue", "first argument"],
            ))
        }
    };

    // キューであることを確認
    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err(fmt_msg(
            MsgKey::MustBeQueue,
            &["queue/enqueue", "first argument"],
        ));
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => Vec::new(),
    };

    let mut new_items = items;
    new_items.push(args[1].clone());

    let mut new_map = queue.clone();
    new_map.insert("items".to_string(), Value::List(new_items));
    Ok(Value::Map(new_map))
}

/// queue/dequeue - キューから要素を取り出し（先頭）
pub fn native_queue_dequeue(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["queue/dequeue"]));
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/dequeue", "argument"])),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/dequeue", "argument"]));
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err(fmt_msg(MsgKey::IsEmpty, &["queue/dequeue", "queue"])),
    };

    if items.is_empty() {
        return Err(fmt_msg(MsgKey::IsEmpty, &["queue/dequeue", "queue"]));
    }

    let item = items[0].clone();
    let new_items: Vec<Value> = items.into_iter().skip(1).collect();

    let mut new_map = queue.clone();
    new_map.insert("items".to_string(), Value::List(new_items));

    // {:value item, :queue new-queue} を返す
    let mut result = HashMap::new();
    result.insert("value".to_string(), item);
    result.insert("queue".to_string(), Value::Map(new_map));
    Ok(Value::Map(result))
}

/// queue/peek - キューの先頭要素を見る（取り出さない）
pub fn native_queue_peek(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["queue/peek"]));
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/peek", "argument"])),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/peek", "argument"]));
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items
        .first()
        .cloned()
        .ok_or_else(|| fmt_msg(MsgKey::IsEmpty, &["queue/peek", "queue"]))
}

/// queue/empty? - キューが空かチェック
pub fn native_queue_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["queue/empty?"]));
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/empty?", "argument"])),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/empty?", "argument"]));
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Bool(true)),
    };

    Ok(Value::Bool(items.is_empty()))
}

/// queue/size - キューのサイズ
pub fn native_queue_size(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["queue/size"]));
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/size", "argument"])),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/size", "argument"]));
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Integer(0)),
    };

    Ok(Value::Integer(items.len() as i64))
}

/// stack/new - 空のスタックを作成
pub fn native_stack_new(_args: &[Value]) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert("type".to_string(), Value::Keyword("stack".to_string()));
    map.insert("items".to_string(), Value::List(Vec::new()));
    Ok(Value::Map(map))
}

/// stack/push - スタックに要素を追加
pub fn native_stack_push(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stack/push"]));
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeStack,
                &["stack/push", "first argument"],
            ))
        }
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err(fmt_msg(
            MsgKey::MustBeStack,
            &["stack/push", "first argument"],
        ));
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => Vec::new(),
    };

    let mut new_items = vec![args[1].clone()];
    new_items.extend(items);

    let mut new_map = stack.clone();
    new_map.insert("items".to_string(), Value::List(new_items));
    Ok(Value::Map(new_map))
}

/// stack/pop - スタックから要素を取り出し
pub fn native_stack_pop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stack/pop"]));
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/pop", "argument"])),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/pop", "argument"]));
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err(fmt_msg(MsgKey::IsEmpty, &["stack/pop", "stack"])),
    };

    if items.is_empty() {
        return Err(fmt_msg(MsgKey::IsEmpty, &["stack/pop", "stack"]));
    }

    let item = items[0].clone();
    let new_items: Vec<Value> = items.into_iter().skip(1).collect();

    let mut new_map = stack.clone();
    new_map.insert("items".to_string(), Value::List(new_items));

    let mut result = HashMap::new();
    result.insert("value".to_string(), item);
    result.insert("stack".to_string(), Value::Map(new_map));
    Ok(Value::Map(result))
}

/// stack/peek - スタックの先頭要素を見る
pub fn native_stack_peek(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stack/peek"]));
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/peek", "argument"])),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/peek", "argument"]));
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items
        .first()
        .cloned()
        .ok_or_else(|| fmt_msg(MsgKey::IsEmpty, &["stack/peek", "stack"]))
}

/// stack/empty? - スタックが空かチェック
pub fn native_stack_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stack/empty?"]));
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/empty?", "argument"])),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/empty?", "argument"]));
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Bool(true)),
    };

    Ok(Value::Bool(items.is_empty()))
}

/// stack/size - スタックのサイズ
pub fn native_stack_size(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stack/size"]));
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/size", "argument"])),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/size", "argument"]));
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Integer(0)),
    };

    Ok(Value::Integer(items.len() as i64))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
pub const FUNCTIONS: &[(&str, fn(&[Value]) -> Result<Value, String>)] = &[
    // Queue functions
    ("queue/new", native_queue_new),
    ("queue/enqueue", native_queue_enqueue),
    ("queue/dequeue", native_queue_dequeue),
    ("queue/peek", native_queue_peek),
    ("queue/empty?", native_queue_empty),
    ("queue/size", native_queue_size),
    // Stack functions
    ("stack/new", native_stack_new),
    ("stack/push", native_stack_push),
    ("stack/pop", native_stack_pop),
    ("stack/peek", native_stack_peek),
    ("stack/empty?", native_stack_empty),
    ("stack/size", native_stack_size),
];
