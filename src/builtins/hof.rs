//! 高階関数

use crate::eval::Evaluator;
use crate::value::Value;

/// map - リストの各要素に関数を適用
pub fn native_map(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("mapは2つの引数が必要です: 実際 {}", args.len()));
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
        _ => Err("mapの第2引数はリストまたはベクタが必要です".to_string()),
    }
}

/// filter - リストから条件を満たす要素を抽出
pub fn native_filter(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("filterは2つの引数が必要です: 実際 {}", args.len()));
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
        _ => Err("filterの第2引数はリストまたはベクタが必要です".to_string()),
    }
}

/// reduce - リストを畳み込み
pub fn native_reduce(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 && args.len() != 3 {
        return Err(format!("reduceは2または3つの引数が必要です: 実際 {}", args.len()));
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
        _ => Err("reduceの第2引数はリストまたはベクタが必要です".to_string()),
    }
}

/// apply - 関数にリストを引数として適用（未使用だが将来のため残す）
#[allow(dead_code)]
pub fn native_apply(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("applyは2つの引数が必要です: 実際 {}", args.len()));
    }

    let func = &args[0];
    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            evaluator.apply_function(func, items)
        }
        _ => Err("applyの第2引数はリストまたはベクタが必要です".to_string()),
    }
}
