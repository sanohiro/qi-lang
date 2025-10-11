//! リスト操作関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// first - リストの最初の要素
pub fn native_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["first"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            Ok(v.first().cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["first", "lists or vectors"])),
    }
}

/// rest - リストの残り
pub fn native_rest(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["rest"]));
    }
    match &args[0] {
        Value::List(v) => {
            if v.is_empty() {
                Ok(Value::List(Vec::new()))
            } else {
                Ok(Value::List(v[1..].to_vec()))
            }
        }
        Value::Vector(v) => {
            if v.is_empty() {
                Ok(Value::Vector(Vec::new()))
            } else {
                Ok(Value::Vector(v[1..].to_vec()))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["rest", "lists or vectors"])),
    }
}

/// len - 長さを取得
pub fn native_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["len"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(Value::Integer(v.len() as i64)),
        Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["len", "strings or collections"])),
    }
}

/// count - 要素数を取得（lenのエイリアス）
pub fn native_count(args: &[Value]) -> Result<Value, String> {
    native_len(args)
}

/// nth - n番目の要素を取得
pub fn native_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["nth"]));
    }
    let index = match &args[1] {
        Value::Integer(n) => *n as usize,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["nth", "an integer"])),
    };
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            Ok(v.get(index).cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["nth", "lists or vectors"])),
    }
}

/// reverse - リストを反転
pub fn native_reverse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["reverse"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut reversed = v.clone();
            reversed.reverse();
            Ok(Value::List(reversed))
        }
        Value::Vector(v) => {
            let mut reversed = v.clone();
            reversed.reverse();
            Ok(Value::Vector(reversed))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["reverse", "lists or vectors"])),
    }
}

/// cons - リストの先頭に要素を追加
pub fn native_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["cons"]));
    }
    match &args[1] {
        Value::Nil => Ok(Value::List(vec![args[0].clone()])),
        Value::List(v) => {
            let mut new_list = vec![args[0].clone()];
            new_list.extend(v.clone());
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = vec![args[0].clone()];
            new_vec.extend(v.clone());
            Ok(Value::List(new_vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["cons", "lists or vectors"])),
    }
}

/// conj - コレクションに要素を追加
pub fn native_conj(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["conj", "2"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut new_list = v.clone();
            for item in &args[1..] {
                new_list.insert(0, item.clone());
            }
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = v.clone();
            new_vec.extend_from_slice(&args[1..]);
            Ok(Value::Vector(new_vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["conj", "lists or vectors"])),
    }
}

/// take - リストの最初のn要素を取得
pub fn native_take(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["take"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["take", "an integer"])),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().take(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().take(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["take", "lists or vectors"])),
    }
}

/// drop - リストの最初のn要素をスキップ
pub fn native_drop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["drop"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["drop", "an integer"])),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().skip(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().skip(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["drop", "lists or vectors"])),
    }
}

/// concat - 複数のリストを連結
pub fn native_concat(args: &[Value]) -> Result<Value, String> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::List(v) | Value::Vector(v) => result.extend(v.clone()),
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["concat", "lists or vectors"])),
        }
    }
    Ok(Value::List(result))
}

/// flatten - ネストしたリストを平坦化
pub fn native_flatten(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["flatten"]));
    }
    fn flatten_value(v: &Value, result: &mut Vec<Value>) {
        match v {
            Value::List(items) | Value::Vector(items) => {
                for item in items {
                    flatten_value(item, result);
                }
            }
            _ => result.push(v.clone()),
        }
    }
    let mut result = Vec::new();
    flatten_value(&args[0], &mut result);
    Ok(Value::List(result))
}

/// range - 0からn-1までのリストを生成
pub fn native_range(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["range"]));
    }
    match &args[0] {
        Value::Integer(n) => {
            let items: Vec<Value> = (0..*n).map(Value::Integer).collect();
            Ok(Value::List(items))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["range", "integers"])),
    }
}

/// last - リストの最後の要素
pub fn native_last(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["last"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            Ok(v.last().cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["last", "lists or vectors"])),
    }
}

/// zip - 2つのリストを組み合わせる
pub fn native_zip(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["zip"]));
    }
    match (&args[0], &args[1]) {
        (Value::List(a), Value::List(b)) | (Value::Vector(a), Value::Vector(b)) => {
            let result: Vec<Value> = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Vector(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["zip", "lists or vectors"])),
    }
}

/// sort - リストをソート
pub fn native_sort(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sort"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            let mut sorted = v.clone();
            sorted.sort_by(|a, b| {
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            Ok(Value::List(sorted))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sort", "lists or vectors"])),
    }
}

/// distinct - 重複を排除
pub fn native_distinct(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["distinct"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            let mut result = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for item in v {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["distinct", "lists or vectors"])),
    }
}

/// take-while - 条件を満たす間要素を取得
pub fn native_take_while(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["take-while", "2", "(predicate, collection)"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut result = Vec::new();
            for item in items {
                let test = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if !test.is_truthy() {
                    break;
                }
                result.push(item.clone());
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["take-while (2nd arg)", "second argument"])),
    }
}

/// drop-while - 条件を満たす間要素をスキップ
pub fn native_drop_while(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["drop-while", "2", "(predicate, collection)"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            let mut dropping = true;
            let mut result = Vec::new();
            for item in items {
                if dropping {
                    let test = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                    if test.is_truthy() {
                        continue;
                    }
                    dropping = false;
                }
                result.push(item.clone());
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["drop-while (2nd arg)", "second argument"])),
    }
}

/// split-at - 指定位置でリストを分割
pub fn native_split_at(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["split-at", "2", "(index, collection)"]));
    }

    let index = match &args[0] {
        Value::Integer(n) => *n as usize,
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["split-at", "index"])),
    };

    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            let split_point = index.min(items.len());
            let left = items[..split_point].to_vec();
            let right = items[split_point..].to_vec();
            Ok(Value::Vector(vec![Value::List(left), Value::List(right)]))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["split-at (2nd arg)", "second argument"])),
    }
}

/// interleave - 2つのリストを交互に組み合わせる
pub fn native_interleave(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["interleave"]));
    }

    let list1 = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["interleave (1st arg)", "first argument"])),
    };

    let list2 = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["interleave (2nd arg)", "second argument"])),
    };

    let mut result = Vec::new();
    let min_len = list1.len().min(list2.len());
    for i in 0..min_len {
        result.push(list1[i].clone());
        result.push(list2[i].clone());
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

            let mut result = HashMap::new();
            for (key, count) in counts {
                result.insert(key, Value::Integer(count));
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["frequencies", "argument"])),
    }
}

/// sort-by - キー関数でソート
pub fn native_sort_by(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["sort-by", "2", "(key-fn, collection)"]));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // 各要素のキーを計算
            let mut keyed: Vec<(Value, Value)> = Vec::new();
            for item in items {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;
                keyed.push((key, item.clone()));
            }

            // キーでソート
            keyed.sort_by(|a, b| {
                match (&a.0, &b.0) {
                    (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    (Value::Integer(x), Value::Float(y)) => {
                        (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });

            let result: Vec<Value> = keyed.into_iter().map(|(_, v)| v).collect();
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["sort-by (2nd arg)", "second argument"])),
    }
}

/// chunk - 固定サイズでリストを分割
pub fn native_chunk(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["chunk", "2", "(size, collection)"]));
    }

    let size = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err(fmt_msg(MsgKey::MustBePositive, &["chunk", "size"])),
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["chunk", "size"])),
    };

    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            let mut result = Vec::new();
            for chunk in items.chunks(size) {
                result.push(Value::List(chunk.to_vec()));
            }
            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["chunk (2nd arg)", "second argument"])),
    }
}

/// max-by - キー関数で最大値を取得
pub fn native_max_by(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["max-by", "2", "(key-fn, collection)"]));
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

            for item in &items[1..] {
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["max-by (2nd arg)", "second argument"])),
    }
}

/// min-by - キー関数で最小値を取得
pub fn native_min_by(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["min-by", "2", "(key-fn, collection)"]));
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

            for item in &items[1..] {
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["min-by (2nd arg)", "second argument"])),
    }
}

/// sum-by - キー関数で合計
pub fn native_sum_by(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["sum-by", "2", "(key-fn, collection)"]));
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["sum-by (2nd arg)", "second argument"])),
    }
}

/// find - 条件を満たす最初の要素を返す
pub fn native_find(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["find", "2", "(predicate, collection)"]));
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["find (2nd arg)", "second argument"])),
    }
}

/// find-index - 条件を満たす最初の要素のインデックスを返す
pub fn native_find_index(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["find-index", "2", "(predicate, collection)"]));
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["find-index (2nd arg)", "second argument"])),
    }
}

/// every? - すべての要素が条件を満たすか
pub fn native_every(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["every?", "2", "(predicate, collection)"]));
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["every? (2nd arg)", "second argument"])),
    }
}

/// some? - いずれかの要素が条件を満たすか
pub fn native_some(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["some?", "2", "(predicate, collection)"]));
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
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["some? (2nd arg)", "second argument"])),
    }
}

/// zipmap - 2つのコレクションからマップを作成
pub fn native_zipmap(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["zipmap", "2", "(keys, vals)"]));
    }

    let keys = match &args[0] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["zipmap (1st arg)", "first argument"])),
    };

    let vals = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["zipmap (2nd arg)", "second argument"])),
    };

    let mut result = std::collections::HashMap::new();
    for (key, val) in keys.iter().zip(vals.iter()) {
        let key_str = match key {
            Value::String(s) => s.clone(),
            Value::Keyword(k) => k.clone(),
            _ => format!("{:?}", key),
        };
        result.insert(key_str, val.clone());
    }

    Ok(Value::Map(result))
}

/// partition-by - 連続する値を述語関数でグループ化
pub fn native_partition_by(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["partition-by", "2", "(predicate, collection)"]));
    }

    let pred_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(Value::List(Vec::new()));
            }

            let mut result: Vec<Value> = Vec::new();
            let mut current_group: Vec<Value> = Vec::new();
            let mut last_pred_result: Option<Value> = None;

            for item in items {
                let pred_result = evaluator.apply_function(pred_fn, std::slice::from_ref(item))?;

                if let Some(ref last) = last_pred_result {
                    // 述語の結果が変わったら新しいグループを開始
                    if !values_equal(last, &pred_result) {
                        if !current_group.is_empty() {
                            result.push(Value::List(current_group.clone()));
                            current_group.clear();
                        }
                    }
                }

                current_group.push(item.clone());
                last_pred_result = Some(pred_result);
            }

            // 最後のグループを追加
            if !current_group.is_empty() {
                result.push(Value::List(current_group));
            }

            Ok(Value::List(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeListOrVector, &["partition-by (2nd arg)", "second argument"])),
    }
}

// 値の等価性チェック用ヘルパー
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}

/// take-nth - n番目ごとの要素を取得
pub fn native_take_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["take-nth", "2", "(n, collection)"]));
    }

    let n = match &args[0] {
        Value::Integer(i) => {
            if *i <= 0 {
                return Err(fmt_msg(MsgKey::MustBePositive, &["take-nth", "n"]));
            }
            *i as usize
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["take-nth", "first argument"])),
    };

    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["take-nth (2nd arg)", "second argument"])),
    };

    let result: Vec<Value> = collection
        .iter()
        .enumerate()
        .filter(|(i, _)| i % n == 0)
        .map(|(_, v)| v.clone())
        .collect();

    Ok(Value::List(result))
}

/// keep - nilを除外してmap
pub fn native_keep(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["keep", "2", "(function, collection)"]));
    }

    let func = &args[0];
    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["keep (2nd arg)", "second argument"])),
    };

    let mut result: Vec<Value> = Vec::new();
    for item in collection {
        let val = evaluator.apply_function(func, std::slice::from_ref(item))?;
        if !matches!(val, Value::Nil) {
            result.push(val);
        }
    }

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
        return Ok(Value::List(Vec::new()));
    }

    let mut result: Vec<Value> = Vec::new();
    let mut last: Option<&Value> = None;

    for item in collection {
        if let Some(prev) = last {
            if !values_equal(prev, item) {
                result.push(item.clone());
            }
        } else {
            result.push(item.clone());
        }
        last = Some(item);
    }

    Ok(Value::List(result))
}

/// drop-last - 最後のn要素を削除
pub fn native_drop_last(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["drop-last", "2", "(n, collection)"]));
    }

    let n = match &args[0] {
        Value::Integer(i) => {
            if *i < 0 {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["drop-last", "n"]));
            }
            *i as usize
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["drop-last", "first argument"])),
    };

    let collection = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["drop-last (2nd arg)", "second argument"])),
    };

    let take_count = if collection.len() > n {
        collection.len() - n
    } else {
        0
    };

    let result: Vec<Value> = collection.iter().take(take_count).cloned().collect();
    Ok(Value::List(result))
}
