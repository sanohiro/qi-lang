//! ストリーム（遅延評価）- メモリ内の無限データ構造

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::sync::Arc;

/// stream - コレクションからストリーム作成
pub fn native_stream(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stream"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let items = items.clone();
            let index = Arc::new(RwLock::new(0));

            let stream = Stream {
                next_fn: Box::new(move || {
                    let mut idx = index.write();
                    if *idx < items.len() {
                        let val = items[*idx].clone();
                        *idx += 1;
                        Some(val)
                    } else {
                        None
                    }
                }),
            };

            Ok(Value::Stream(Arc::new(RwLock::new(stream))))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["stream", "argument"])),
    }
}

/// range-stream - 範囲ストリーム作成
pub fn native_range_stream(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["range-stream"]));
    }

    let start = match &args[0] {
        Value::Integer(n) => *n,
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["range-stream", "start"])),
    };

    let end = match &args[1] {
        Value::Integer(n) => *n,
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["range-stream", "end"])),
    };

    let current = Arc::new(RwLock::new(start));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut cur = current.write();
            if *cur < end {
                let val = *cur;
                *cur += 1;
                Some(Value::Integer(val))
            } else {
                None
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// iterate - 無限ストリーム作成（関数の反復適用）
pub fn native_iterate(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["iterate"]));
    }

    let func = args[0].clone();
    let current = Arc::new(RwLock::new(args[1].clone()));
    let evaluator = evaluator.clone();

    let stream = Stream {
        next_fn: Box::new(move || {
            let val = current.read().clone();
            if let Ok(next) = evaluator.apply_function(&func, &[val.clone()]) {
                *current.write() = next;
            }
            Some(val)
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// repeat - 同じ値の無限ストリーム
pub fn native_repeat(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["repeat"]));
    }

    let val = args[0].clone();

    let stream = Stream {
        next_fn: Box::new(move || Some(val.clone())),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// cycle - リストを循環する無限ストリーム
pub fn native_cycle(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["cycle"]));
    }

    let items = match &args[0] {
        Value::List(items) | Value::Vector(items) => items.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["cycle", "argument"])),
    };

    if items.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["cycle", "argument"]));
    }

    let index = Arc::new(RwLock::new(0));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut idx = index.write();
            let val = items[*idx].clone();
            *idx = (*idx + 1) % items.len();
            Some(val)
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// stream-map - ストリームの各要素に関数を適用
pub fn native_stream_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stream-map"]));
    }

    let func = args[0].clone();
    let source_stream = match &args[1] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["stream-map (2nd arg)", "a stream"])),
    };

    let evaluator = evaluator.clone();

    let stream = Stream {
        next_fn: Box::new(move || {
            let next_val = {
                let stream = source_stream.read();
                (stream.next_fn)()
            };
            next_val.and_then(|v| evaluator.apply_function(&func, &[v]).ok())
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// stream-filter - ストリームの要素をフィルタ
pub fn native_stream_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stream-filter"]));
    }

    let pred = args[0].clone();
    let source_stream = match &args[1] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["stream-filter (2nd arg)", "a stream"])),
    };

    let evaluator = evaluator.clone();

    let stream = Stream {
        next_fn: Box::new(move || loop {
            let next_val = {
                let stream = source_stream.read();
                (stream.next_fn)()
            };
            match next_val {
                None => return None,
                Some(v) => {
                    if let Ok(result) = evaluator.apply_function(&pred, &[v.clone()]) {
                        if result.is_truthy() {
                            return Some(v);
                        }
                    }
                }
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// stream-take - ストリームの最初のn個を取得
pub fn native_stream_take(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stream-take"]));
    }

    let n = match &args[0] {
        Value::Integer(n) if *n >= 0 => *n as usize,
        _ => return Err(fmt_msg(MsgKey::MustBeNonNegative, &["stream-take", "first argument"])),
    };

    let source_stream = match &args[1] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["stream-take (2nd arg)", "a stream"])),
    };

    let count = Arc::new(RwLock::new(0));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut cnt = count.write();
            if *cnt < n {
                *cnt += 1;
                let stream = source_stream.read();
                (stream.next_fn)()
            } else {
                None
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// stream-drop - ストリームの最初のn個をスキップ
pub fn native_stream_drop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stream-drop"]));
    }

    let n = match &args[0] {
        Value::Integer(n) if *n >= 0 => *n as usize,
        _ => return Err(fmt_msg(MsgKey::MustBeNonNegative, &["stream-drop", "first argument"])),
    };

    let source_stream = match &args[1] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["stream-drop (2nd arg)", "a stream"])),
    };

    let skipped = Arc::new(RwLock::new(false));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut sk = skipped.write();
            if !*sk {
                for _ in 0..n {
                    let stream = source_stream.read();
                    (stream.next_fn)()?;
                }
                *sk = true;
            }
            drop(sk);
            let stream = source_stream.read();
            (stream.next_fn)()
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// realize - ストリームをリストに変換（実行）
pub fn native_realize(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["realize"]));
    }

    let stream = match &args[0] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["realize", "a stream"])),
    };

    let mut result = Vec::new();
    loop {
        let next_val = {
            let s = stream.read();
            (s.next_fn)()
        };
        match next_val {
            Some(v) => result.push(v),
            None => break,
        }
    }

    Ok(Value::List(result))
}
