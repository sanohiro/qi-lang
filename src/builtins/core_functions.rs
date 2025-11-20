//! Core高階関数
//!
//! 関数基礎（5個）: identity, constantly, partial, comp, apply

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::sync::Arc;

/// identity - 引数をそのまま返す
pub fn native_identity(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["identity"]));
    }
    Ok(args[0].clone())
}

/// constantly - 常に同じ値を返す関数を生成
pub fn native_constantly(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["constantly"]));
    }
    let value = args[0].clone();
    // 単純に値を返すだけの関数を作る（評価時に特別処理）
    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "_",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy(
            crate::eval::hof_keys::CONSTANTLY_VALUE,
        )),
        env: {
            let mut env = crate::value::Env::new();
            env.set(crate::eval::hof_keys::CONSTANTLY_VALUE, value);
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: false,
        has_special_processing: false,
    })))
}

/// partial - 部分適用
pub fn native_partial(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["partial", "2+", "(function and at least 1 arg)"],
        ));
    }

    let func = args[0].clone();
    let partial_args: im::Vector<Value> = args[1..].iter().cloned().collect();

    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "&rest",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy(
            crate::eval::hof_keys::PARTIAL_PLACEHOLDER,
        )),
        env: {
            let mut env = crate::value::Env::new();
            env.set(crate::eval::hof_keys::PARTIAL_FUNC, func);
            env.set(
                crate::eval::hof_keys::PARTIAL_ARGS.to_string(),
                Value::List(partial_args),
            );
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: true,
        has_special_processing: true,
    })))
}

/// comp - 関数合成（右から左に適用）
///
/// 注意: この関数はEvaluatorを必要とするため、mod.rsでの登録時に特別な処理が必要です
pub fn native_comp(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["comp", "1"]));
    }

    // 1つの関数の場合はそのまま返す
    if args.len() == 1 {
        return Ok(args[0].clone());
    }

    // 複数の関数の場合は合成された関数を返す
    let funcs: im::Vector<Value> = args.iter().cloned().collect();
    Ok(Value::Function(Arc::new(crate::value::Function {
        params: vec![crate::value::Pattern::Var(crate::intern::intern_symbol(
            "x",
        ))],
        body: Arc::new(crate::value::Expr::symbol_dummy("__comp_placeholder__")),
        env: {
            let mut env = crate::value::Env::new();
            env.set(
                crate::eval::hof_keys::COMP_FUNCS.to_string(),
                Value::List(funcs),
            );
            Arc::new(parking_lot::RwLock::new(env))
        },
        is_variadic: false,
        has_special_processing: true,
    })))
}

/// apply - リストを引数として関数適用
///
/// 注意: この関数はEvaluatorを必要とするため、mod.rsでの登録時に特別な処理が必要です
pub fn native_apply(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["apply", "2", "(function and list)"],
        ));
    }

    let func = &args[0];
    match &args[1] {
        Value::List(items) | Value::Vector(items) => {
            // im::Vector を &[Value] に変換
            let items_vec: Vec<Value> = items.iter().cloned().collect();
            evaluator.apply_function(func, &items_vec)
        }
        _ => Err(fmt_msg(
            MsgKey::MustBeListOrVector,
            &["apply (2nd arg)", "second argument"],
        )),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category core/functions
/// @qi-doc:functions identity, constantly, partial, comp, apply
///
/// 注意: comp, applyはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("identity", native_identity),
    ("constantly", native_constantly),
    ("partial", native_partial),
];
