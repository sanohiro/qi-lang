//! リスト操作 - 分割・フィルタ関数

use super::helpers::values_equal;
use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// partition-by - 連続する値を述語関数でグループ化
pub fn native_partition_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["partition-by", "2", "(predicate, collection)"],
        ));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(Value::List(im::Vector::new()));
            }

            // im::Vector直接構築（中間Vec排除）
            let mut result = im::Vector::new();
            let mut current_group = im::Vector::new();
            let mut last_pred_result: Option<Value> = None;

            for item in items {
                let pred_result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;

                if let Some(ref last) = last_pred_result {
                    // 述語の結果が変わったら新しいグループを開始
                    if !values_equal(last, &pred_result) && !current_group.is_empty() {
                        result.push_back(Value::List(current_group.clone()));
                        current_group = im::Vector::new();
                    }
                }

                current_group.push_back(item.clone());
                last_pred_result = Some(pred_result);
            }

            // 最後のグループを追加
            if !current_group.is_empty() {
                result.push_back(Value::List(current_group));
            }

            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["partition-by (2nd arg)", "second argument"],
        )),
    }
}

/// keep - nilを除外してmap
pub fn native_keep(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["keep", "2", "(function, collection)"],
        ));
    }

    let func = &args[0];
    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["keep (2nd arg)", "second argument"],
            ))
        }
    };

    // im::Vector直接構築（中間Vec排除）
    let mut result = im::Vector::new();
    for item in collection {
        let val = evaluator.apply_function(func, std::slice::from_ref(item))?;
        if !matches!(val, Value::Nil) {
            result.push_back(val);
        }
    }

    Ok(Value::List(result))
}
