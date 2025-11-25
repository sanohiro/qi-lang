//! 集合演算関数
//!
//! このモジュールは `std-set` feature でコンパイルされます。

// ValueはArc<RwLock<Env>>を含むため、clippyがmutable_key_typeを警告する。
// しかし、我々のHash/Eq実装は一貫しており（RwLockの中身は比較しない）、
// 実際にはHashSetのキーとして安全に使用できる。
#![allow(clippy::mutable_key_type)]

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashSet;

/// ハッシュ化できない値が含まれているかチェック
fn check_hashable(items: &im::Vector<Value>) -> Result<(), String> {
    for item in items {
        match item {
            Value::Float(_) => {
                return Err(fmt_msg(
                    MsgKey::SetOperationError,
                    &["Float values cannot be used in set operations"],
                ))
            }
            Value::Function(_)
            | Value::NativeFunc(_)
            | Value::Macro(_)
            | Value::Atom(_)
            | Value::Channel(_)
            | Value::Scope(_)
            | Value::Stream(_)
            | Value::Uvar(_) => {
                return Err(fmt_msg(
                    MsgKey::SetOperationError,
                    &[&format!(
                        "{} cannot be used in set operations",
                        item.type_name()
                    )],
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

/// union - 和集合
pub fn native_union(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(vec![].into()));
    }

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                check_hashable(items)?;
                for item in items {
                    if seen.insert(item.clone()) {
                        result.push(item.clone());
                    }
                }
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::AllElementsMustBe,
                    &["union", "lists or vectors"],
                ))
            }
        }
    }

    Ok(Value::List(result.into()))
}

/// intersect - 積集合
pub fn native_intersect(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["intersect", "2"]));
    }

    // 最初のリストをベースにする
    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["intersect", "lists or vectors"],
            ))
        }
    };

    check_hashable(first)?;
    let mut result: HashSet<Value> = first.iter().cloned().collect();

    // 他のリストとの積集合を取る
    for arg in &args[1..] {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                check_hashable(items)?;
                let set: HashSet<Value> = items.iter().cloned().collect();
                result.retain(|k| set.contains(k));
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::AllElementsMustBe,
                    &["intersect", "lists or vectors"],
                ))
            }
        }
    }

    // 元の値を復元（順序を保持）
    let values: Vec<Value> = first
        .iter()
        .filter(|v| result.contains(v))
        .cloned()
        .collect();

    Ok(Value::List(values.into()))
}

/// difference - 差集合（第1引数から第2引数以降を除く）
pub fn native_difference(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["difference", "2"]));
    }

    let first = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["difference", "lists or vectors"],
            ))
        }
    };

    check_hashable(first)?;
    let mut exclude = HashSet::new();
    for arg in &args[1..] {
        match arg {
            Value::List(items) | Value::Vector(items) => {
                check_hashable(items)?;
                for item in items {
                    exclude.insert(item.clone());
                }
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::AllElementsMustBe,
                    &["difference", "lists or vectors"],
                ))
            }
        }
    }

    let values: Vec<Value> = first
        .iter()
        .filter(|v| !exclude.contains(v))
        .cloned()
        .collect();

    Ok(Value::List(values.into()))
}

/// subset? - 部分集合判定
pub fn native_subset(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "subset?");

    let subset = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["subset?", "lists or vectors"],
            ))
        }
    };

    let superset = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["subset?", "lists or vectors"],
            ))
        }
    };

    check_hashable(subset)?;
    check_hashable(superset)?;

    let superset_set: HashSet<Value> = superset.iter().cloned().collect();

    for item in subset {
        if !superset_set.contains(item) {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}

/// superset? - 上位集合判定（第1引数が第2引数の上位集合か）
pub fn native_superset(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "superset?");

    // superset?(A, B) = subset?(B, A)
    native_subset(&[args[1].clone(), args[0].clone()])
}

/// disjoint? - 互いに素判定（共通要素がないか）
pub fn native_disjoint(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "disjoint?");

    let set1 = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["disjoint?", "lists or vectors"],
            ))
        }
    };

    let set2 = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["disjoint?", "lists or vectors"],
            ))
        }
    };

    check_hashable(set1)?;
    check_hashable(set2)?;

    let set1_hash: HashSet<Value> = set1.iter().cloned().collect();

    for item in set2 {
        if set1_hash.contains(item) {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}

/// symmetric-difference - 対称差（どちらか一方にのみ存在する要素）
pub fn native_symmetric_difference(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "symmetric-difference");

    let set1 = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["symmetric-difference", "lists or vectors"],
            ))
        }
    };

    let set2 = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["symmetric-difference", "lists or vectors"],
            ))
        }
    };

    check_hashable(set1)?;
    check_hashable(set2)?;

    let set1_hash: HashSet<Value> = set1.iter().cloned().collect();
    let set2_hash: HashSet<Value> = set2.iter().cloned().collect();

    let mut result = Vec::new();
    let mut seen = HashSet::new();

    // set1にのみ存在する要素
    for item in set1 {
        if !set2_hash.contains(item) && seen.insert(item.clone()) {
            result.push(item.clone());
        }
    }

    // set2にのみ存在する要素
    for item in set2 {
        if !set1_hash.contains(item) && seen.insert(item.clone()) {
            result.push(item.clone());
        }
    }

    Ok(Value::List(result.into()))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category set
/// @qi-doc:functions union, intersection, difference, subset?, superset?
pub const FUNCTIONS: super::NativeFunctions = &[
    ("set/union", native_union),
    ("set/intersect", native_intersect),
    ("set/difference", native_difference),
    ("set/subset?", native_subset),
    ("set/superset?", native_superset),
    ("set/disjoint?", native_disjoint),
    ("set/symmetric-difference", native_symmetric_difference),
];
