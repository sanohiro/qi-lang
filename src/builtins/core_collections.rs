//! Coreコレクション操作関数
//!
//! リスト基本（29個）: first, rest, last, nth, take, drop, map, filter, reduce, pmap, cons, conj,
//!                     concat, flatten, range, repeat, reverse, zip, sort, distinct, find, every, some,
//!                     take-while, drop-while, len, count, split-at, interleave
//! マップ基本（9個）: get, keys, vals, assoc, dissoc, merge, get-in, update-in, update
//!
//! 注: map, filter, reduce, pmap, take-while, drop-while, find, every, some, update-in, update
//!     は Evaluator が必要なため、mod.rs で別途エクスポートされます

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

// ========================================
// リスト操作（Evaluator不要）
// ========================================

/// first - リストの最初の要素
pub fn native_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["first"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(v.head().cloned().unwrap_or(Value::Nil)),
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
                Ok(Value::List(im::Vector::new()))
            } else {
                Ok(Value::List(v.iter().skip(1).cloned().collect()))
            }
        }
        Value::Vector(v) => {
            if v.is_empty() {
                Ok(Value::Vector(im::Vector::new()))
            } else {
                Ok(Value::Vector(v.iter().skip(1).cloned().collect()))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["rest", "lists or vectors"])),
    }
}

/// last - リストの最後の要素
pub fn native_last(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["last"]));
    }
    match &args[0] {
        Value::List(v) | Value::Vector(v) => Ok(v.last().cloned().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["last", "lists or vectors"])),
    }
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
        Value::List(v) | Value::Vector(v) => Ok(v.get(index).cloned().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["nth", "lists or vectors"])),
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
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["len", "strings or collections"],
        )),
    }
}

/// count - 要素数を取得（lenのエイリアス）
pub fn native_count(args: &[Value]) -> Result<Value, String> {
    native_len(args)
}

/// cons - リストの先頭に要素を追加
/// im::Vectorの構造共有を活かしてO(1)で実行
/// 戻り値は第二引数（コレクション）の型を維持します
pub fn native_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["cons"]));
    }
    match &args[1] {
        Value::Nil => Ok(Value::List(vec![args[0].clone()].into())),
        Value::List(v) => {
            let mut new_list = v.clone();
            new_list.push_front(args[0].clone());
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = v.clone();
            new_vec.push_front(args[0].clone());
            Ok(Value::Vector(new_vec))
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
            for item in args[1..].iter().rev() {
                new_list.push_front(item.clone());
            }
            Ok(Value::List(new_list))
        }
        Value::Vector(v) => {
            let mut new_vec = v.clone();
            for item in &args[1..] {
                new_vec.push_back(item.clone());
            }
            Ok(Value::Vector(new_vec))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["conj", "lists or vectors"])),
    }
}

/// concat - 複数のリストを連結
/// 戻り値は第一引数の型を優先します（引数がない場合はListを返す）
pub fn native_concat(args: &[Value]) -> Result<Value, String> {
    // 第一引数の型を記憶
    let result_type = if args.is_empty() {
        None
    } else {
        match &args[0] {
            Value::List(_) => Some(false),  // List
            Value::Vector(_) => Some(true), // Vector
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["concat", "lists or vectors"])),
        }
    };

    let mut result = im::Vector::new();
    for arg in args {
        if let Some(iter) = arg.as_sequence_iter() {
            result.extend(iter.cloned());
        } else {
            return Err(fmt_msg(MsgKey::TypeOnly, &["concat", "lists or vectors"]));
        }
    }

    match result_type {
        Some(true) => Ok(Value::Vector(result)),
        _ => Ok(Value::List(result)),
    }
}

/// flatten - ネストしたリストを平坦化
pub fn native_flatten(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["flatten"]));
    }
    fn flatten_value(v: &Value, result: &mut im::Vector<Value>) {
        if let Some(iter) = v.as_sequence_iter() {
            for item in iter {
                flatten_value(item, result);
            }
        } else {
            result.push_back(v.clone());
        }
    }
    let mut result = im::Vector::new();
    flatten_value(&args[0], &mut result);
    Ok(Value::List(result))
}

/// range - 範囲の生成
/// (range n) => [0 1 2 ... n-1]
/// (range start end) => [start start+1 ... end-1]
pub fn native_range(args: &[Value]) -> Result<Value, String> {
    match args.len() {
        1 => {
            // (range n) => [0 1 2 ... n-1]
            match &args[0] {
                Value::Integer(n) => {
                    let items: im::Vector<Value> = (0..*n).map(Value::Integer).collect();
                    Ok(Value::List(items))
                }
                _ => Err(fmt_msg(MsgKey::TypeOnly, &["range", "integers"])),
            }
        }
        2 => {
            // (range start end) => [start start+1 ... end-1]
            match (&args[0], &args[1]) {
                (Value::Integer(start), Value::Integer(end)) => {
                    let items: im::Vector<Value> = (*start..*end).map(Value::Integer).collect();
                    Ok(Value::List(items))
                }
                _ => Err(fmt_msg(MsgKey::TypeOnly, &["range", "integers"])),
            }
        }
        _ => Err(fmt_msg(
            MsgKey::ArgCountMismatch,
            &["1 or 2", &args.len().to_string()],
        )),
    }
}

/// repeat - 同じ値をn回繰り返したリストを生成
/// 引数: (n val) - 繰り返し回数と値
/// 例: (repeat 5 0) => [0 0 0 0 0]
pub fn native_repeat(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["repeat"]));
    }

    match &args[0] {
        Value::Integer(n) => {
            if *n < 0 {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["repeat", "count"]));
            }
            let val = args[1].clone();
            let items: im::Vector<Value> = (0..*n).map(|_| val.clone()).collect();
            Ok(Value::List(items))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["repeat (count)", "integer"])),
    }
}

/// reverse - リストを反転
pub fn native_reverse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["reverse"]));
    }
    match &args[0] {
        Value::List(v) => {
            let reversed: im::Vector<Value> = v.iter().rev().cloned().collect();
            Ok(Value::List(reversed))
        }
        Value::Vector(v) => {
            let reversed: im::Vector<Value> = v.iter().rev().cloned().collect();
            Ok(Value::Vector(reversed))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["reverse", "lists or vectors"])),
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

/// sort - リストをソート
/// 戻り値は入力コレクションの型を維持します
pub fn native_sort(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sort"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut sorted: Vec<Value> = v.iter().cloned().collect();
            sorted.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::List(sorted.into()))
        }
        Value::Vector(v) => {
            let mut sorted: Vec<Value> = v.iter().cloned().collect();
            sorted.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::Vector(sorted.into()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sort", "lists or vectors"])),
    }
}

/// distinct - 重複を排除
/// 戻り値は入力コレクションの型を維持します
pub fn native_distinct(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["distinct"]));
    }
    match &args[0] {
        Value::List(v) => {
            let mut result = im::Vector::new();
            let mut seen = std::collections::HashSet::new();
            for item in v {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push_back(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        Value::Vector(v) => {
            let mut result = im::Vector::new();
            let mut seen = std::collections::HashSet::new();
            for item in v {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push_back(item.clone());
                }
            }
            Ok(Value::Vector(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["distinct", "lists or vectors"])),
    }
}

/// zip - 2つのリストを組み合わせる（List/Vectorの混在も可）
/// 戻り値は第一引数の型を優先します
pub fn native_zip(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["zip"]));
    }

    // 第一引数の型を記憶
    let is_first_vector = matches!(&args[0], Value::Vector(_));

    // 両引数からim::Vectorを取得
    let vec_a = match &args[0] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["zip", "lists or vectors"])),
    };
    let vec_b = match &args[1] {
        Value::List(v) | Value::Vector(v) => v,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["zip", "lists or vectors"])),
    };

    let result: im::Vector<Value> = vec_a
        .iter()
        .zip(vec_b.iter())
        .map(|(x, y)| Value::Vector(vec![x.clone(), y.clone()].into()))
        .collect();

    if is_first_vector {
        Ok(Value::Vector(result))
    } else {
        Ok(Value::List(result))
    }
}

// ========================================
// マップ操作（Evaluator不要）
// ========================================

/// get - マップから値を取得
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["get"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let key = args[1].to_map_key()?;
            Ok(m.get(&key).cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["get", "a map"])),
    }
}

/// keys - マップのキーを取得
/// マップのキーは内部的に文字列で格納されているため、元の型に復元する
pub fn native_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["keys"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let keys: im::Vector<Value> = m
                .keys()
                .map(|k| match k {
                    crate::value::MapKey::Keyword(kw) => Value::Keyword(kw.clone()),
                    crate::value::MapKey::Symbol(sym) => Value::Symbol(sym.clone()),
                    crate::value::MapKey::String(s) => Value::String(s.clone()),
                    crate::value::MapKey::Integer(n) => Value::Integer(*n),
                })
                .collect();
            Ok(Value::List(keys))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["keys", "maps"])),
    }
}

/// vals - マップの値を取得
pub fn native_vals(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["vals"]));
    }
    match &args[0] {
        Value::Map(m) => {
            let vals: im::Vector<Value> = m.values().cloned().collect();
            Ok(Value::List(vals))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["vals", "maps"])),
    }
}

/// assoc - マップに新しいキー・値のペアを追加
pub fn native_assoc(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Err(msg(MsgKey::AssocMapAndKeyValues).to_string());
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_map = m.clone();
            for i in (1..args.len()).step_by(2) {
                let key = args[i].to_map_key()?;
                new_map.insert(key, args[i + 1].clone());
            }
            Ok(Value::Map(new_map))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["assoc", "a map"])),
    }
}

/// dissoc - マップから指定したキーを削除
pub fn native_dissoc(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(msg(MsgKey::DissocMapAndKeys).to_string());
    }
    match &args[0] {
        Value::Map(m) => {
            let mut new_map = m.clone();

            // 削除するキーのリストを作成
            for arg in &args[1..] {
                let key = arg.to_map_key()?;
                new_map = new_map.without(&key);
            }

            Ok(Value::Map(new_map))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["dissoc", "a map"])),
    }
}

/// merge - 複数のマップをマージ
pub fn native_merge(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["merge", "1"]));
    }
    let mut result = crate::new_hashmap();
    for arg in args {
        match arg {
            Value::Map(m) => {
                for (k, v) in m {
                    result.insert(k.clone(), v.clone());
                }
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["merge", "maps"])),
        }
    }
    Ok(Value::Map(result))
}

/// get-in - ネストしたマップから値を取得
pub fn native_get_in(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["get-in", "2 or 3", "(map, path, default?)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["get-in", "path"])),
    };
    let default = if args.len() == 3 {
        args[2].clone()
    } else {
        Value::Nil
    };

    let mut current = map.clone();
    for key_val in path {
        let key = key_val.to_map_key()?;

        match current {
            Value::Map(m) => {
                current = m.get(&key).cloned().unwrap_or(Value::Nil);
                if matches!(current, Value::Nil) {
                    return Ok(default);
                }
            }
            _ => return Ok(default),
        }
    }

    Ok(current)
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category core/collections
/// @qi-doc:functions first, rest, last, nth, len, count, cons, conj, concat, reverse, sort, distinct, range, repeat, take, drop, zip, etc.
pub const FUNCTIONS: super::NativeFunctions = &[
    // リスト操作
    ("first", native_first),
    ("rest", native_rest),
    ("last", native_last),
    ("nth", native_nth),
    ("len", native_len),
    ("count", native_count),
    ("cons", native_cons),
    ("conj", native_conj),
    ("concat", native_concat),
    ("flatten", native_flatten),
    ("range", native_range),
    ("repeat", native_repeat),
    ("reverse", native_reverse),
    ("take", native_take),
    ("drop", native_drop),
    ("sort", native_sort),
    ("distinct", native_distinct),
    ("zip", native_zip),
    // マップ操作
    ("get", native_get),
    ("keys", native_keys),
    ("vals", native_vals),
    ("assoc", native_assoc),
    ("dissoc", native_dissoc),
    ("merge", native_merge),
    ("get-in", native_get_in),
];
