//! 高階関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// map - リストの各要素に関数を適用
/// 戻り値は入力コレクションの型を維持します
pub fn native_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["map"]));
    }

    let func = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) => {
            // Vec一時利用でwith_capacity最適化→im::Vectorへ変換
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                let result = evaluator.apply_function(func, std::slice::from_ref(item))?;
                results.push(result);
            }
            Ok(Value::List(results.into()))
        }
        Value::Vector(items) => {
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                let result = evaluator.apply_function(func, std::slice::from_ref(item))?;
                results.push(result);
            }
            Ok(Value::Vector(results.into()))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["map (2nd arg)", "lists or vectors"],
        )),
    }
}

/// filter - リストから条件を満たす要素を抽出
/// 戻り値は入力コレクションの型を維持します
pub fn native_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["filter"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) => {
            let mut results = im::Vector::new();
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    results.push_back(item.clone());
                }
            }
            Ok(Value::List(results))
        }
        Value::Vector(items) => {
            let mut results = im::Vector::new();
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    results.push_back(item.clone());
                }
            }
            Ok(Value::Vector(results))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["filter (2nd arg)", "lists or vectors"],
        )),
    }
}

/// reduce - リストを畳み込み
pub fn native_reduce(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
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

            for item in items.iter().skip(start_idx) {
                acc = evaluator.apply_function(func, &[acc, item.clone()])?;
            }
            Ok(acc)
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["reduce (2nd arg)", "lists or vectors"],
        )),
    }
}

/// pmap - 並列map
///
/// コレクションの各要素に関数を並列適用します。
/// 全ての関数型（NativeFunc、ユーザー定義関数）を完全に並列化します。
pub fn native_pmap(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    use rayon::prelude::*;

    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["pmap"]));
    }

    let func = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // 小さいコレクション（< 100要素）は通常のmapにフォールバック
            // 並列処理のオーバーヘッドが変換コストで相殺されるため
            if items.len() < 100 {
                return native_map(args, evaluator);
            }

            // im::Vectorはpar_iterをサポートしていないため、一時的にVecに変換
            let mut items_vec = Vec::with_capacity(items.len());
            for item in items.iter() {
                items_vec.push(item.clone());
            }

            // すべての関数を並列処理（Evaluatorが&selfなので複数スレッドで共有可能）
            let results: Result<Vec<_>, _> = items_vec
                .par_iter()
                .map(|item| evaluator.apply_function(func, std::slice::from_ref(item)))
                .collect();

            match collection {
                Value::List(_) => Ok(Value::List(results?.into())),
                Value::Vector(_) => Ok(Value::Vector(results?.into())),
                _ => unreachable!(),
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["pmap (2nd arg)", "lists or vectors"],
        )),
    }
}

/// each - コレクションの各要素に関数を適用（副作用用）
/// mapと異なり、戻り値は収集せずnilを返します
pub fn native_each(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["each"]));
    }

    let func = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            for item in items {
                evaluator.apply_function(func, std::slice::from_ref(item))?;
            }
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["each (2nd arg)", "lists or vectors"],
        )),
    }
}

/// pfilter - 並列filter
///
/// コレクションから条件を満たす要素を並列抽出します。
pub fn native_pfilter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    use rayon::prelude::*;

    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["pfilter"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // 小さいコレクション（< 100要素）は通常のfilterにフォールバック
            if items.len() < 100 {
                return native_filter(args, evaluator);
            }

            // im::Vectorはpar_iterをサポートしていないため、一時的にVecに変換
            let mut items_vec = Vec::with_capacity(items.len());
            for item in items.iter() {
                items_vec.push(item.clone());
            }

            // 並列でフィルタリング
            let results: Result<Vec<_>, _> = items_vec
                .par_iter()
                .filter_map(|item| {
                    match evaluator.apply_function(pred, std::slice::from_ref(item)) {
                        Ok(result) if result.is_truthy() => Some(Ok(item.clone())),
                        Ok(_) => None,
                        Err(e) => Some(Err(e)),
                    }
                })
                .collect();

            match collection {
                Value::List(_) => Ok(Value::List(results?.into())),
                Value::Vector(_) => Ok(Value::Vector(results?.into())),
                _ => unreachable!(),
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["pfilter (2nd arg)", "lists or vectors"],
        )),
    }
}

/// preduce - 並列reduce
///
/// コレクションを並列畳み込みします。
/// 結合法則を満たす演算（+, *, max等）でのみ正しい結果が得られます。
///
/// 引数順序: (preduce fn collection init) - reduceと同じ
pub fn native_preduce(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    use rayon::prelude::*;

    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["preduce", "3"]));
    }

    let func = &args[0];
    let collection = &args[1];
    let init = args[2].clone();

    match collection {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Ok(init);
            }

            // 小さいコレクション（< 100要素）は通常のreduceにフォールバック
            if items.len() < 100 {
                return native_reduce(args, evaluator);
            }

            // im::Vectorはpar_iterをサポートしていないため、一時的にVecに変換
            let mut items_vec = Vec::with_capacity(items.len());
            for item in items.iter() {
                items_vec.push(item.clone());
            }

            // 並列reduce
            items_vec
                .par_iter()
                .try_fold(
                    || init.clone(),
                    |acc, item| evaluator.apply_function(func, &[acc, item.clone()]),
                )
                .try_reduce(
                    || init.clone(),
                    |a, b| evaluator.apply_function(func, &[a, b]),
                )
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["preduce (2nd arg)", "lists or vectors"],
        )),
    }
}

/// partition - 述語でリストを2つに分割
pub fn native_partition(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["partition"]));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // Vec一時利用（容量推定: 50:50分布と仮定）→im::Vectorへ変換
            let half = items.len() / 2;
            let mut truthy = Vec::with_capacity(half);
            let mut falsy = Vec::with_capacity(half);
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                if result.is_truthy() {
                    truthy.push(item.clone());
                } else {
                    falsy.push(item.clone());
                }
            }
            Ok(Value::Vector(
                vec![Value::List(truthy.into()), Value::List(falsy.into())].into(),
            ))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["partition (2nd arg)", "lists or vectors"],
        )),
    }
}

/// group-by - キー関数でリストをグループ化
pub fn native_group_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["group-by"]));
    }

    let key_fn = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // グループ数の推定: items.len() / 4（4個に1個が新しいグループと仮定）
            let estimated_groups = (items.len() / 4).max(4);
            let mut groups: std::collections::HashMap<String, im::Vector<Value>> =
                std::collections::HashMap::with_capacity(estimated_groups);
            for item in items {
                let key = evaluator.apply_function(key_fn, std::slice::from_ref(item))?;
                // Valueから純粋な文字列を抽出（Displayの引用符を避けるため）
                let key_str = match &key {
                    Value::String(s) => s.clone(),
                    Value::Keyword(k) => format!(":{}", k),
                    Value::Symbol(s) => s.to_string(),
                    _ => format!("{}", key),
                };
                groups.entry(key_str).or_default().push_back(item.clone());
            }

            let mut result = crate::new_hashmap();
            for (key_str, values) in groups {
                result.insert(key_str, Value::List(values));
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["group-by (2nd arg)", "lists or vectors"],
        )),
    }
}

/// map-lines - 文字列の各行に関数を適用
pub fn native_map_lines(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["map-lines"]));
    }

    let func = &args[0];
    let text = &args[1];

    match text {
        Value::String(s) => {
            let lines: Vec<&str> = s.lines().collect();
            let mut results = Vec::with_capacity(lines.len());
            for line in lines {
                let result = evaluator.apply_function(func, &[Value::String(line.to_string())])?;
                if let Value::String(transformed) = result {
                    results.push(transformed);
                } else {
                    return Err(fmt_msg(
                        MsgKey::FuncMustReturnType,
                        &["map-lines", "string"],
                    ));
                }
            }
            Ok(Value::String(results.join("\n")))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["map-lines (2nd arg)", "strings"],
        )),
    }
}

/// update - マップの値を関数で更新
pub fn native_update(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update", "3", "(map, key, fn)"],
        ));
    }

    let map = &args[0];
    let key_val = &args[1];
    let func = &args[2];

    match map {
        Value::Map(m) => {
            let key = match key_val {
                Value::String(s) => s.clone(),
                Value::Keyword(k) => k.to_string(),
                _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
            };

            let current_value = m.get(&key).cloned().unwrap_or(Value::Nil);
            let new_value = evaluator.apply_function(func, &[current_value])?;

            let mut result = m.clone();
            result.insert(key, new_value);
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["update", "a map"])),
    }
}

/// update-in - ネストしたマップの値を関数で更新
pub fn native_update_in(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["update-in", "3", "(map, path, fn)"],
        ));
    }

    let map = &args[0];
    let path = match &args[1] {
        Value::List(p) | Value::Vector(p) => p,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["update-in", "path"])),
    };
    let func = &args[2];

    if path.is_empty() {
        return Err(fmt_msg(MsgKey::MustNotBeEmpty, &["update-in", "path"]));
    }

    match map {
        Value::Map(m) => {
            let mut result = m.clone();
            update_in_helper(&mut result, path, 0, func, evaluator)?;
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(MsgKey::MustBeMap, &["update-in", "first argument"])),
    }
}

fn update_in_helper(
    map: &mut crate::HashMap<String, Value>,
    path: &im::Vector<Value>,
    index: usize,
    func: &Value,
    evaluator: &Evaluator,
) -> Result<(), String> {
    let key = match &path[index] {
        Value::String(s) => s.clone(),
        Value::Keyword(k) => k.to_string(),
        _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
    };

    if index == path.len() - 1 {
        // 最後のキー：値を更新
        let current_value = map.get(&key).cloned().unwrap_or(Value::Nil);
        let new_value = evaluator.apply_function(func, &[current_value])?;
        map.insert(key, new_value);
    } else {
        // 中間のキー：再帰的に処理
        let next_val = map
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Value::Map(crate::new_hashmap()));
        match next_val {
            Value::Map(mut inner_map) => {
                update_in_helper(&mut inner_map, path, index + 1, func, evaluator)?;
                map.insert(key, Value::Map(inner_map));
            }
            _ => {
                // 既存の値がマップでない場合は上書き
                let mut new_map = crate::new_hashmap();
                update_in_helper(&mut new_map, path, index + 1, func, evaluator)?;
                map.insert(key, Value::Map(new_map));
            }
        }
    }
    Ok(())
}

/// count-by - 述語でカウント
pub fn native_count_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["count-by", "2", "(predicate, collection)"],
        ));
    }

    let pred = &args[0];
    let collection = &args[1];

    match collection {
        Value::List(items) | Value::Vector(items) => {
            // true/falseの2値なので容量2で十分
            let mut counts: std::collections::HashMap<String, i64> =
                std::collections::HashMap::with_capacity(2);
            for item in items {
                let result = evaluator.apply_function(pred, std::slice::from_ref(item))?;
                let key = if result.is_truthy() { "true" } else { "false" };
                *counts.entry(key.to_string()).or_insert(0) += 1;
            }

            let mut result = crate::new_hashmap();
            for (key, count) in counts {
                result.insert(key, Value::Integer(count));
            }
            Ok(Value::Map(result))
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["count-by (2nd arg)", "second argument"],
        )),
    }
}

/// complement - 述語の否定
pub fn native_complement(args: &[Value]) -> Result<Value, String> {
    use std::sync::Arc;

    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["complement", "1", "(function)"],
        ));
    }

    let func = args[0].clone();

    // 引数を否定する関数を返す
    // 実装はeval.rsのapply_funcで特殊処理される
    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "x",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy("x")),
        env: {
            let mut env = crate::value::Env::new();
            env.set(crate::eval::hof_keys::COMPLEMENT_FUNC, func);
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: false,
        has_special_processing: true,
    })))
}

/// juxt - 複数関数を並列適用
pub fn native_juxt(args: &[Value]) -> Result<Value, String> {
    use std::sync::Arc;

    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["juxt", "1"]));
    }

    let funcs = args.to_vec();

    // 実装はeval.rsのapply_funcで特殊処理される
    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "x",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy("x")),
        env: {
            let mut env = crate::value::Env::new();
            env.set(
                crate::eval::hof_keys::JUXT_FUNCS.to_string(),
                Value::List(funcs.into()),
            );
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: false,
        has_special_processing: true,
    })))
}

/// tap> - 副作用タップ（流れを止めずに観察）- 高階関数版
///
/// Unixのteeコマンド相当の機能。
/// データフローを止めずに副作用（ログ、デバッグ出力など）を実行します。
///
/// 使用例:
/// ```qi
/// (data
///  |> process
///  |> ((fn/tap> print))  ;; データを表示しつつ通過させる
///  |> save)
/// ```
pub fn native_tap(args: &[Value]) -> Result<Value, String> {
    use std::sync::Arc;

    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["fn/tap>"]));
    }

    let func = args[0].clone();

    // (fn [x] (do (f x) x)) を返す
    // 実装はeval.rsのapply_funcで特殊処理される
    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "x",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy("x")),
        env: {
            let mut env = crate::value::Env::new();
            env.set(crate::eval::hof_keys::TAP_FUNC, func);
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: false,
        has_special_processing: true,
    })))
}

/// tap - 副作用タップ（Evaluator版、パイプライン内で直接使用可能）
///
/// `(tap print)`の形で使えるバージョン。
/// パイプライン内で`|> (tap print)`と書ける。
///
/// 使用例:
/// ```qi
/// ([1 2 3]
///  |> (map inc)
///  |> (tap print)  ;; 括弧1つでOK
///  |> sum)
/// ```
pub fn native_tap_direct(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["tap"]));
    }

    let func = &args[0];
    let value = &args[1];

    // 副作用関数を実行（結果は無視）
    let _ = evaluator.apply_function(func, std::slice::from_ref(value));

    // 元の値をそのまま返す
    Ok(value.clone())
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category fn
/// @qi-doc:functions map, filter, reduce, pmap, pfilter, preduce, partition, group-by, map-lines, update, update-in, count-by, complement, juxt, tap>, tap
///
/// 注意: map, filter, reduce, pmap, pfilter, preduce, partition, group_by,
/// map_lines, update, update_in, count_by, tap_directはEvaluatorが必要なため、
/// mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("fn/complement", native_complement),
    ("fn/juxt", native_juxt),
    ("fn/tap>", native_tap),
];
