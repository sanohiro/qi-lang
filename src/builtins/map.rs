//! マップ操作関数
//!
//! 注: 基本的なマップ操作(get, keys, vals, assoc, dissoc, merge, get-in)は
//! core_collections.rsで実装されています。
//! このモジュールには高度なマップ操作のみを含みます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// select-keys - マップから指定したキーのみ選択
pub fn native_select_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["select-keys"]));
    }
    match (&args[0], &args[1]) {
        (Value::Map(m), Value::List(keys) | Value::Vector(keys)) => {
            let mut result = crate::new_hashmap();
            for key_val in keys {
                let key = key_val.to_map_key()?;
                if let Some(v) = m.get(&key) {
                    result.insert(key, v.clone());
                }
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["select-keys", "map and list/vector"],
        )),
    }
}

/// assoc-in - ネストしたマップに値を設定
pub fn native_assoc_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["assoc-in", "3", "(map, path, value)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["assoc-in", "path"])),
    };
    let value = &args[2];

    if path.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["assoc-in", "path"]));
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            assoc_in_helper(&mut result, path, 0, value)?;
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeMap, &["assoc-in", "first argument"])),
    }
}

fn assoc_in_helper(
    map: &mut crate::HashMap<crate::value::MapKey, Value>,
    path: &im::Vector<Value>,
    index: usize,
    value: &Value,
) -> Result<(), String> {
    if index >= path.len() {
        return Err(fmt_msg(
            MsgKey::PathIndexOutOfBounds,
            &["assoc-in", &index.to_string(), &path.len().to_string()],
        ));
    }
    let key = path[index].to_map_key()?;

    if index == path.len() - 1 {
        // 最後のキー：値を設定
        map.insert(key, value.clone());
    } else {
        // 中間のキー：再帰的に処理
        let next_val = map
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Value::Map(crate::new_hashmap()));
        match next_val {
            Value::Map(mut inner_map) => {
                assoc_in_helper(&mut inner_map, path, index + 1, value)?;
                map.insert(key, Value::Map(inner_map));
            }
            _ => {
                // 既存の値がマップでない場合は上書き
                let mut new_map = crate::new_hashmap();
                assoc_in_helper(&mut new_map, path, index + 1, value)?;
                map.insert(key, Value::Map(new_map));
            }
        }
    }
    Ok(())
}

/// dissoc-in - ネストしたマップからキーを削除
pub fn native_dissoc_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["dissoc-in", "2", "(map, path)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["dissoc-in", "path"])),
    };

    if path.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["dissoc-in", "path"]));
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            dissoc_in_helper(&mut result, path, 0)?;
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeMap, &["dissoc-in", "first argument"])),
    }
}

fn dissoc_in_helper(
    map: &mut crate::HashMap<crate::value::MapKey, Value>,
    path: &im::Vector<Value>,
    index: usize,
) -> Result<(), String> {
    if index >= path.len() {
        return Err(fmt_msg(
            MsgKey::PathIndexOutOfBounds,
            &["dissoc-in", &index.to_string(), &path.len().to_string()],
        ));
    }
    let key = path[index].to_map_key()?;

    if index == path.len() - 1 {
        // 最後のキー：削除
        map.remove(&key);
    } else {
        // 中間のキー：再帰的に処理
        if let Some(Value::Map(inner_map)) = map.get_mut(&key) {
            let mut inner_clone = inner_map.clone();
            dissoc_in_helper(&mut inner_clone, path, index + 1)?;
            map.insert(key, Value::Map(inner_clone));
        }
    }
    Ok(())
}

/// update-keys - マップのすべてのキーに関数を適用
pub fn native_update_keys(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update-keys", "2", "(key-fn, map)"],
        ));
    }

    let key_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = crate::new_hashmap();
            for (k, v) in m {
                let key_val = Value::String(k.to_string());
                let new_key_val = evaluator.apply_function(key_fn, &[key_val])?;
                let new_key = match new_key_val {
                    Value::String(s) => crate::value::MapKey::String(s),
                    Value::Keyword(kw) => crate::value::MapKey::Keyword(kw),
                    _ => crate::value::MapKey::String(format!("{:?}", new_key_val)),
                };
                result.insert(new_key, v.clone());
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeMap,
            &["update-keys", "second argument"],
        )),
    }
}

/// update-vals - マップのすべての値に関数を適用
pub fn native_update_vals(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update-vals", "2", "(val-fn, map)"],
        ));
    }

    let val_fn = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = crate::new_hashmap();
            for (k, v) in m {
                let new_val = evaluator.apply_function(val_fn, std::slice::from_ref(v))?;
                result.insert(k.clone(), new_val);
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeMap,
            &["update-vals", "second argument"],
        )),
    }
}

/// filter-vals - 述語関数で値をフィルタリング
///
/// 引数:
/// - pred: 述語関数（値を受け取りboolを返す）
/// - map: マップ
///
/// 戻り値:
/// - 述語を満たす値のみを含む新しいマップ
///
/// 例:
/// ```qi
/// (map/filter-vals (fn [v] (> v 18)) {:alice 25 :bob 17 :charlie 30})
/// ;=> {:alice 25 :charlie 30}
/// ```
pub fn native_filter_vals(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["filter-vals", "2", "(pred, map)"],
        ));
    }

    let pred = &args[0];
    let map_val = &args[1];

    match map_val {
        Value::Map(m) => {
            let mut result = crate::new_hashmap();
            for (k, v) in m {
                let pred_result = evaluator.apply_function(pred, std::slice::from_ref(v))?;
                if pred_result.is_truthy() {
                    result.insert(k.clone(), v.clone());
                }
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeMap,
            &["filter-vals", "second argument"],
        )),
    }
}

/// group-by - キー関数でコレクションをグループ化
///
/// 引数:
/// - key-fn: キー関数（要素からグルーピングキーを生成）
/// - coll: コレクション（リストまたはベクタ）
///
/// 戻り値:
/// - キーごとにグループ化されたマップ（値はリスト）
///
/// 例:
/// ```qi
/// (map/group-by :type [{:type "A" :val 1} {:type "A" :val 2} {:type "B" :val 3}])
/// ;=> {"A" [{:type "A" :val 1} {:type "A" :val 2}] "B" [{:type "B" :val 3}]}
///
/// (map/group-by (fn [x] (mod x 2)) [1 2 3 4 5 6])
/// ;=> {0 [2 4 6] 1 [1 3 5]}
/// ```
pub fn native_group_by(
    args: &[Value],
    evaluator: &crate::eval::Evaluator,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["group-by", "2", "(key-fn, coll)"],
        ));
    }

    let key_fn = &args[0];
    let coll = match &args[1] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["group-by", "second argument"],
            ))
        }
    };

    let mut groups: std::collections::HashMap<crate::value::MapKey, Vec<Value>> =
        std::collections::HashMap::new();

    for item in coll {
        let key_val = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;
        // ValueをMapKeyに変換（to_map_keyがサポートする型のみ）
        // Bool/Float/Nilはエラーになるため、文字列化する
        let key = match key_val.to_map_key() {
            Ok(k) => k,
            Err(_) => {
                // Bool, Float, Nil等はMapKeyに変換できないため、文字列化
                // TODO: MapKeyにBool/Float/Nil型を追加すれば元の型を保持できる
                crate::value::MapKey::String(format!("{}", key_val))
            }
        };
        groups.entry(key).or_default().push(item.clone());
    }

    // HashMap<MapKey, Vec<Value>> -> HashMap<MapKey, Value::List>
    let result: crate::HashMap<crate::value::MapKey, Value> = groups
        .into_iter()
        .map(|(k, v)| (k, Value::List(v.into())))
        .collect();

    Ok(Value::Map(result))
}

/// deep-merge - ネストしたマップを再帰的にマージ
///
/// 引数:
/// - map1, map2, ...: マージするマップ（可変長引数）
///
/// 戻り値:
/// - 再帰的にマージされた新しいマップ
///
/// 動作:
/// - ネストしたマップは再帰的にマージ
/// - 非マップ値は後のマップの値で上書き
/// - 空の引数リストの場合は空マップを返す
///
/// 例:
/// ```qi
/// (map/deep-merge {:a {:b 1}} {:a {:c 2}})
/// ;=> {:a {:b 1 :c 2}}
///
/// (map/deep-merge {:a {:b 1}} {:a {:b 2 :c 3}} {:a {:d 4}})
/// ;=> {:a {:b 2 :c 3 :d 4}}
/// ```
pub fn native_deep_merge(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Map(crate::new_hashmap()));
    }

    let mut result = crate::new_hashmap();

    for arg in args {
        match arg {
            Value::Map(m) => {
                deep_merge_into(&mut result, m);
            }
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["deep-merge", "maps"])),
        }
    }

    Ok(Value::Map(result))
}

/// deep-mergeのヘルパー関数：targetにsourceをマージ
fn deep_merge_into(
    target: &mut crate::HashMap<crate::value::MapKey, Value>,
    source: &crate::HashMap<crate::value::MapKey, Value>,
) {
    for (key, value) in source {
        match (target.get(key), value) {
            (Some(Value::Map(target_map)), Value::Map(source_map)) => {
                // 両方ともマップの場合：再帰的にマージ
                let mut merged = target_map.clone();
                deep_merge_into(&mut merged, source_map);
                target.insert(key.clone(), Value::Map(merged));
            }
            _ => {
                // それ以外の場合：sourceの値で上書き
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category map
/// @qi-doc:functions select-keys, assoc-in, dissoc-in, deep-merge
///
/// 注意: update-keys, update-vals, filter-vals, group-byはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("map/select-keys", native_select_keys),
    ("map/assoc-in", native_assoc_in),
    ("map/dissoc-in", native_dissoc_in),
    ("map/deep-merge", native_deep_merge),
];
