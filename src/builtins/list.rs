//! リスト操作関数

use crate::i18n::{fmt_msg, msg, MsgKey};
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
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["first"])),
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
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["rest"])),
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
        _ => Err(fmt_msg(MsgKey::StringOrCollectionOnly, &["len"])),
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
        _ => return Err(msg(MsgKey::NthSecondArgInteger).to_string()),
    };
    match &args[0] {
        Value::List(v) | Value::Vector(v) => {
            Ok(v.get(index).cloned().unwrap_or(Value::Nil))
        }
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["nth"])),
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
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["reverse"])),
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
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["cons"])),
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
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["conj"])),
    }
}

/// take - リストの最初のn要素を取得
pub fn native_take(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["take"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(msg(MsgKey::TakeFirstArgInteger).to_string()),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().take(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().take(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["take"])),
    }
}

/// drop - リストの最初のn要素をスキップ
pub fn native_drop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["drop"]));
    }
    let n = match &args[0] {
        Value::Integer(i) => *i as usize,
        _ => return Err(msg(MsgKey::DropFirstArgInteger).to_string()),
    };
    match &args[1] {
        Value::List(v) => Ok(Value::List(v.iter().skip(n).cloned().collect())),
        Value::Vector(v) => Ok(Value::Vector(v.iter().skip(n).cloned().collect())),
        _ => Err(fmt_msg(MsgKey::ListOrVectorOnly, &["drop"])),
    }
}

/// concat - 複数のリストを連結
pub fn native_concat(args: &[Value]) -> Result<Value, String> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::List(v) | Value::Vector(v) => result.extend(v.clone()),
            _ => return Err(msg(MsgKey::ConcatListOrVectorOnly).to_string()),
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
        _ => Err(msg(MsgKey::RangeIntegerOnly).to_string()),
    }
}
