//! リスト操作 - 集約・統計関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// sort-by - キー関数でソート
pub fn native_sort_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["sort-by", "2", "(key-fn, collection)"],
        ));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // 各要素のキーを計算（容量事前確保）
            let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
            for item in items {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;
                keyed.push((key, item.clone()));
            }

            // キーでソート
            keyed.sort_by(|a, b| match (&a.0, &b.0) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                (Value::Integer(x), Value::Float(y)) => (*x as f64)
                    .partial_cmp(y)
                    .unwrap_or(std::cmp::Ordering::Equal),
                (Value::Float(x), Value::Integer(y)) => x
                    .partial_cmp(&(*y as f64))
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            });

            let result: Vec<Value> = keyed.into_iter().map(|(_, v)| v).collect();
            Ok(Value::List(result.into()))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["sort-by (2nd arg)", "second argument"],
        )),
    }
}

/// max-by - キー関数で最大値を取得
pub fn native_max_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["max-by", "2", "(key-fn, collection)"],
        ));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(Value::Nil);
            }

            let mut max_item = &items[0];
            let mut max_key = evaluator.apply_function(key_fn, std::slice::from_ref(max_item))?;

            for item in items.iter().skip(1) {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;

                let is_greater = match (&key, &max_key) {
                    (Value::Integer(k), Value::Integer(m)) => k > m,
                    (Value::Float(k), Value::Float(m)) => k > m,
                    (Value::Integer(k), Value::Float(m)) => (*k as f64) > *m,
                    (Value::Float(k), Value::Integer(m)) => *k > (*m as f64),
                    (Value::String(k), Value::String(m)) => k > m,
                    _ => false,
                };

                if is_greater {
                    max_item = item;
                    max_key = key;
                }
            }

            Ok(max_item.clone())
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["max-by (2nd arg)", "second argument"],
        )),
    }
}

/// min-by - キー関数で最小値を取得
pub fn native_min_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["min-by", "2", "(key-fn, collection)"],
        ));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(Value::Nil);
            }

            let mut min_item = &items[0];
            let mut min_key = evaluator.apply_function(key_fn, std::slice::from_ref(min_item))?;

            for item in items.iter().skip(1) {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;

                let is_less = match (&key, &min_key) {
                    (Value::Integer(k), Value::Integer(m)) => k < m,
                    (Value::Float(k), Value::Float(m)) => k < m,
                    (Value::Integer(k), Value::Float(m)) => (*k as f64) < *m,
                    (Value::Float(k), Value::Integer(m)) => *k < (*m as f64),
                    (Value::String(k), Value::String(m)) => k < m,
                    _ => false,
                };

                if is_less {
                    min_item = item;
                    min_key = key;
                }
            }

            Ok(min_item.clone())
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["min-by (2nd arg)", "second argument"],
        )),
    }
}

/// sum-by - キー関数で合計
pub fn native_sum_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["sum-by", "2", "(key-fn, collection)"],
        ));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut int_sum: i64 = 0;
            let mut float_sum: f64 = 0.0;
            let mut has_float = false;

            for item in items {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;

                match key {
                    Value::Integer(n) => {
                        if has_float {
                            float_sum += n as f64;
                        } else {
                            int_sum += n;
                        }
                    }
                    Value::Float(f) => {
                        if !has_float {
                            float_sum = int_sum as f64;
                            has_float = true;
                        }
                        float_sum += f;
                    }
                    _ => return Err(fmt_msg(MsgKey::FuncMustReturnType, &["sum-by", "numbers"])),
                }
            }

            if has_float {
                Ok(Value::Float(float_sum))
            } else {
                Ok(Value::Integer(int_sum))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["sum-by (2nd arg)", "second argument"],
        )),
    }
}
