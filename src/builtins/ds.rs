//! データ構造 - Queue, Stack等

use crate::check_args;
use crate::builtins::util::{convert_string_map_to_mapkey, kw};
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;

/// queue/new - 空のキューを作成
pub fn native_queue_new(_args: &[Value]) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert(
        ":type".to_string(),
        Value::Keyword(crate::intern::intern_keyword("queue")),
    );
    map.insert(":items".to_string(), Value::List(Vec::new().into()));
    Ok(Value::Map(convert_string_map_to_mapkey(map)))
}

/// queue/enqueue - 要素をキューに追加（末尾）
pub fn native_queue_enqueue(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "queue/enqueue");

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
    if !matches!(queue.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "queue") {
        return Err(fmt_msg(
            MsgKey::MustBeQueue,
            &["queue/enqueue", "first argument"],
        ));
    }

    let items = match queue.get(&kw("items")) {
        Some(Value::List(lst)) => lst.clone(),
        _ => Vec::new().into(),
    };

    let mut new_items = items;
    new_items.push_back(args[1].clone());

    let mut new_map = queue.clone();
    new_map.insert(kw("items"), Value::List(new_items));
    Ok(Value::Map(new_map))
}

/// queue/dequeue - キューから要素を取り出し（先頭）
pub fn native_queue_dequeue(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "queue/dequeue");

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/dequeue", "argument"])),
    };

    if !matches!(queue.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/dequeue", "argument"]));
    }

    let items = match queue.get(&kw("items")) {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err(fmt_msg(MsgKey::IsEmpty, &["queue/dequeue", "queue"])),
    };

    if items.is_empty() {
        return Err(fmt_msg(MsgKey::IsEmpty, &["queue/dequeue", "queue"]));
    }

    let item = items[0].clone();
    let new_items: Vec<Value> = items.into_iter().skip(1).collect();

    let mut new_map = queue.clone();
    new_map.insert(kw("items"), Value::List(new_items.into()));

    // {:value item, :queue new-queue} を返す
    let mut result = HashMap::new();
    result.insert(":value".to_string(), item);
    result.insert(":queue".to_string(), Value::Map(new_map));
    Ok(Value::Map(convert_string_map_to_mapkey(result)))
}

/// queue/peek - キューの先頭要素を見る（取り出さない）
pub fn native_queue_peek(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "queue/peek");

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/peek", "argument"])),
    };

    if !matches!(queue.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/peek", "argument"]));
    }

    let items = match queue.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items
        .iter()
        .next()
        .cloned()
        .ok_or_else(|| fmt_msg(MsgKey::IsEmpty, &["queue/peek", "queue"]))
}

/// queue/empty? - キューが空かチェック
pub fn native_queue_empty(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "queue/empty?");

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/empty?", "argument"])),
    };

    if !matches!(queue.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/empty?", "argument"]));
    }

    let items = match queue.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Bool(true)),
    };

    Ok(Value::Bool(items.is_empty()))
}

/// queue/size - キューのサイズ
pub fn native_queue_size(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "queue/size");

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/size", "argument"])),
    };

    if !matches!(queue.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "queue") {
        return Err(fmt_msg(MsgKey::MustBeQueue, &["queue/size", "argument"]));
    }

    let items = match queue.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Integer(0)),
    };

    Ok(Value::Integer(items.len() as i64))
}

/// stack/new - 空のスタックを作成
pub fn native_stack_new(_args: &[Value]) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert(
        ":type".to_string(),
        Value::Keyword(crate::intern::intern_keyword("stack")),
    );
    map.insert(":items".to_string(), Value::List(Vec::new().into()));
    Ok(Value::Map(convert_string_map_to_mapkey(map)))
}

/// stack/push - スタックに要素を追加
pub fn native_stack_push(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "stack/push");

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeStack,
                &["stack/push", "first argument"],
            ))
        }
    };

    if !matches!(stack.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "stack") {
        return Err(fmt_msg(
            MsgKey::MustBeStack,
            &["stack/push", "first argument"],
        ));
    }

    let items = match stack.get(&kw("items")) {
        Some(Value::List(lst)) => lst.clone(),
        _ => Vec::new().into(),
    };

    let mut new_items: im::Vector<Value> = vec![args[1].clone()].into();
    new_items.append(items);

    let mut new_map = stack.clone();
    new_map.insert(kw("items"), Value::List(new_items));
    Ok(Value::Map(new_map))
}

/// stack/pop - スタックから要素を取り出し
pub fn native_stack_pop(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "stack/pop");

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/pop", "argument"])),
    };

    if !matches!(stack.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/pop", "argument"]));
    }

    let items = match stack.get(&kw("items")) {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err(fmt_msg(MsgKey::IsEmpty, &["stack/pop", "stack"])),
    };

    if items.is_empty() {
        return Err(fmt_msg(MsgKey::IsEmpty, &["stack/pop", "stack"]));
    }

    let item = items[0].clone();
    let new_items: Vec<Value> = items.into_iter().skip(1).collect();

    let mut new_map = stack.clone();
    new_map.insert(kw("items"), Value::List(new_items.into()));

    let mut result = HashMap::new();
    result.insert(":value".to_string(), item);
    result.insert(":stack".to_string(), Value::Map(new_map));
    Ok(Value::Map(convert_string_map_to_mapkey(result)))
}

/// stack/peek - スタックの先頭要素を見る
pub fn native_stack_peek(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "stack/peek");

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/peek", "argument"])),
    };

    if !matches!(stack.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/peek", "argument"]));
    }

    let items = match stack.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items
        .iter()
        .next()
        .cloned()
        .ok_or_else(|| fmt_msg(MsgKey::IsEmpty, &["stack/peek", "stack"]))
}

/// stack/empty? - スタックが空かチェック
pub fn native_stack_empty(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "stack/empty?");

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/empty?", "argument"])),
    };

    if !matches!(stack.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/empty?", "argument"]));
    }

    let items = match stack.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Bool(true)),
    };

    Ok(Value::Bool(items.is_empty()))
}

/// stack/size - スタックのサイズ
pub fn native_stack_size(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "stack/size");

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeStack, &["stack/size", "argument"])),
    };

    if !matches!(stack.get(&kw("type")), Some(Value::Keyword(k)) if &**k == "stack") {
        return Err(fmt_msg(MsgKey::MustBeStack, &["stack/size", "argument"]));
    }

    let items = match stack.get(&kw("items")) {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Integer(0)),
    };

    Ok(Value::Integer(items.len() as i64))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category ds
/// @qi-doc:functions queue/new, queue/enqueue, queue/dequeue, queue/peek, queue/empty?, queue/size, stack/new, stack/push, stack/pop, stack/peek, stack/empty?, stack/size
pub const FUNCTIONS: super::NativeFunctions = &[
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
