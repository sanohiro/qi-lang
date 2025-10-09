//! 高階関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// map - リストの各要素に関数を適用
pub fn native_map(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["map"]));
    }

    let func = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut results = Vec::new();
            for item in items {
                let result = evaluator.apply_function(func, &[item.clone()])?;
                results.push(result);
            }
            Ok(Value::List(results))
        }
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["map (2nd arg)"])),
    }
}

/// filter - リストから条件を満たす要素を抽出
pub fn native_filter(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["filter"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut results = Vec::new();
            for item in items {
                let result = evaluator.apply_function(pred, &[item.clone()])?;
                if result.is_truthy() {
                    results.push(item.clone());
                }
            }
            Ok(Value::List(results))
        }
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["filter (2nd arg)"])),
    }
}

/// reduce - リストを畳み込み
pub fn native_reduce(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 && args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["reduce"]));
    }

    let func = &args[0];
    let collection = &args[1];
    let init = if args.len() == 3 {
        Some(args[2].clone())
    } else {
        None
    };

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(init.unwrap_or(Value::Nil));
            }

            let (start_idx, mut acc) = if let Some(initial) = init {
                (0, initial)
            } else {
                (1, items[0].clone())
            };

            for item in &items[start_idx..] {
                acc = evaluator.apply_function(func, &[acc, item.clone()])?;
            }
            Ok(acc)
        }
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["reduce (2nd arg)"])),
    }
}

/// apply - 関数にリストを引数として適用（未使用だが将来のため残す）
#[allow(dead_code)]
pub fn native_apply(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["apply"]));
    }

    let func = &args[0];
    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            evaluator.apply_function(func, items)
        }
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["apply (2nd arg)"])),
    }
}
