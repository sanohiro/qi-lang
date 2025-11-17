//! リスト操作 - 述語・検索関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// take-while - 条件を満たす間要素を取得
pub fn native_take_while(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["take-while", "2", "(predicate, collection)"],
        ));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // im::Vectorを直接使用（中間Vec排除）
            let mut result = im::Vector::new();
            for item in items {
                let test = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if !test.is_truthy() {
                    break;
                }
                result.push_back(item.clone());
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["take-while (2nd arg)", "second argument"],
        )),
    }
}

/// drop-while - 条件を満たす間要素をスキップ
pub fn native_drop_while(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["drop-while", "2", "(predicate, collection)"],
        ));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // im::Vectorを直接使用（中間Vec排除）
            let mut dropping = true;
            let mut result = im::Vector::new();
            for item in items {
                if dropping {
                    let test = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                    if test.is_truthy() {
                        continue;
                    }
                    dropping = false;
                }
                result.push_back(item.clone());
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["drop-while (2nd arg)", "second argument"],
        )),
    }
}

/// find - 条件を満たす最初の要素を返す
pub fn native_find(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["find", "2", "(predicate, collection)"],
        ));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            for item in items {
                let result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    return Ok(item.clone());
                }
            }
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["find (2nd arg)", "second argument"],
        )),
    }
}

/// find-index - 条件を満たす最初の要素のインデックスを返す
pub fn native_find_index(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["find-index", "2", "(predicate, collection)"],
        ));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            for (idx, item) in items.iter().enumerate() {
                let result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    return Ok(Value::Integer(idx as i64));
                }
            }
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["find-index (2nd arg)", "second argument"],
        )),
    }
}

/// list/every? - すべての要素が条件を満たすか
pub fn native_every(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["list/every?", "2", "(predicate, collection)"],
        ));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            for item in items {
                let result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;
                if !result.is_truthy() {
                    return Ok(Value::Bool(false));
                }
            }
            Ok(Value::Bool(true))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["list/every? (2nd arg)", "second argument"],
        )),
    }
}

/// list/some? - いずれかの要素が条件を満たすか
pub fn native_some(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["list/some?", "2", "(predicate, collection)"],
        ));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            for item in items {
                let result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["list/some? (2nd arg)", "second argument"],
        )),
    }
}
