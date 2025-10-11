//! データ構造 - Queue, Stack等

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
        return Err("queue/enqueue: requires 2 arguments (queue item)".to_string());
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("queue/enqueue: first argument must be a queue".to_string()),
    };

    // キューであることを確認
    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err("queue/enqueue: first argument must be a queue".to_string());
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
        return Err("queue/dequeue: requires 1 argument (queue)".to_string());
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("queue/dequeue: argument must be a queue".to_string()),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err("queue/dequeue: argument must be a queue".to_string());
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err("queue/dequeue: queue is empty".to_string()),
    };

    if items.is_empty() {
        return Err("queue/dequeue: queue is empty".to_string());
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
        return Err("queue/peek: requires 1 argument (queue)".to_string());
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("queue/peek: argument must be a queue".to_string()),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err("queue/peek: argument must be a queue".to_string());
    }

    let items = match queue.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items.first().cloned().ok_or_else(|| "queue/peek: queue is empty".to_string())
}

/// queue/empty? - キューが空かチェック
pub fn native_queue_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("queue/empty?: requires 1 argument (queue)".to_string());
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("queue/empty?: argument must be a queue".to_string()),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err("queue/empty?: argument must be a queue".to_string());
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
        return Err("queue/size: requires 1 argument (queue)".to_string());
    }

    let queue = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("queue/size: argument must be a queue".to_string()),
    };

    if !matches!(queue.get("type"), Some(Value::Keyword(k)) if k == "queue") {
        return Err("queue/size: argument must be a queue".to_string());
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
        return Err("stack/push: requires 2 arguments (stack item)".to_string());
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("stack/push: first argument must be a stack".to_string()),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err("stack/push: first argument must be a stack".to_string());
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
        return Err("stack/pop: requires 1 argument (stack)".to_string());
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("stack/pop: argument must be a stack".to_string()),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err("stack/pop: argument must be a stack".to_string());
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst.clone(),
        _ => return Err("stack/pop: stack is empty".to_string()),
    };

    if items.is_empty() {
        return Err("stack/pop: stack is empty".to_string());
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
        return Err("stack/peek: requires 1 argument (stack)".to_string());
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("stack/peek: argument must be a stack".to_string()),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err("stack/peek: argument must be a stack".to_string());
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Nil),
    };

    items.first().cloned().ok_or_else(|| "stack/peek: stack is empty".to_string())
}

/// stack/empty? - スタックが空かチェック
pub fn native_stack_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("stack/empty?: requires 1 argument (stack)".to_string());
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("stack/empty?: argument must be a stack".to_string()),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err("stack/empty?: argument must be a stack".to_string());
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
        return Err("stack/size: requires 1 argument (stack)".to_string());
    }

    let stack = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("stack/size: argument must be a stack".to_string()),
    };

    if !matches!(stack.get("type"), Some(Value::Keyword(k)) if k == "stack") {
        return Err("stack/size: argument must be a stack".to_string());
    }

    let items = match stack.get("items") {
        Some(Value::List(lst)) => lst,
        _ => return Ok(Value::Integer(0)),
    };

    Ok(Value::Integer(items.len() as i64))
}
