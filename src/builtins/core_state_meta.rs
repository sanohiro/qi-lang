//! Core状態管理・メタプログラミング関数
//!
//! 状態管理（4個）: atom, deref, swap!, reset!
//! メタ（4個）: eval, uvar, variable, macro?
//! 合計8個のCore関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::parser::Parser;
use crate::value::Value;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// グローバルなuvarカウンター
static UVAR_COUNTER: AtomicU64 = AtomicU64::new(0);

// ========================================
// 状態管理関数（4個）
// ========================================

/// atom - アトムを作成
pub fn native_atom(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["atom"]));
    }

    Ok(Value::Atom(Arc::new(RwLock::new(args[0].clone()))))
}

/// deref - アトムから値を取得
pub fn native_deref(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["deref"]));
    }

    match &args[0] {
        Value::Atom(a) => Ok(a.read().clone()),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["deref", "atoms"])),
    }
}

/// reset! - アトムの値を直接セット
pub fn native_reset(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["reset!"]));
    }

    match &args[0] {
        Value::Atom(a) => {
            let new_value = args[1].clone();
            *a.write() = new_value.clone();
            Ok(new_value)
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["reset!", "an atom"])),
    }
}

/// swap! - 関数を適用してアトムをアトミックに更新
pub fn native_swap(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["swap!", "2"]));
    }

    match &args[0] {
        Value::Atom(a) => {
            let current_value = a.read().clone();
            let func = &args[1];

            // 関数に現在の値と追加の引数を渡す
            let mut func_args = vec![current_value];
            func_args.extend_from_slice(&args[2..]);

            let new_value = evaluator.apply_function(func, &func_args)?;
            *a.write() = new_value.clone();
            Ok(new_value)
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["swap!", "an atom"])),
    }
}

// ========================================
// メタプログラミング関数（4個）
// ========================================

/// uvar - 一意な変数を生成
pub fn native_uvar(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["uvar"]));
    }

    let id = UVAR_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Uvar(id))
}

/// variable - 変数かどうかをチェック
pub fn native_variable(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["variable"]));
    }

    let is_var = match &args[0] {
        Value::Symbol(s) => {
            // nil, true, false は変数ではない
            !matches!(s.as_str(), "nil" | "true" | "false")
        }
        Value::Uvar(_) => true,
        _ => false,
    };

    Ok(Value::Bool(is_var))
}

/// macro? - マクロかどうかをチェック
pub fn native_macro_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["macro?"]));
    }

    Ok(Value::Bool(matches!(&args[0], Value::Macro(_))))
}

/// eval - 式を評価
pub fn native_eval(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["eval"]));
    }

    match &args[0] {
        Value::String(code) => {
            // 文字列の場合はパースして評価
            let mut parser = Parser::new(code).map_err(|e| format!("eval: {}", e))?;
            let expr = parser.parse().map_err(|e| format!("eval: {}", e))?;
            evaluator.eval(&expr)
        }
        // データ構造の場合は直接評価できるよう変換が必要
        // とりあえず文字列化して再パース（簡易実装）
        value => {
            let code = format!("{}", value);
            let mut parser = Parser::new(&code).map_err(|e| format!("eval: {}", e))?;
            let expr = parser.parse().map_err(|e| format!("eval: {}", e))?;
            evaluator.eval(&expr)
        }
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category core/state-meta
/// @qi-doc:functions atom, deref, swap!, reset!, eval, uvar, variable, macro?
///
/// 注意: swap!, evalはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("atom", native_atom),
    ("deref", native_deref),
    ("reset!", native_reset),
    ("uvar", native_uvar),
    ("variable", native_variable),
    ("macro?", native_macro_q),
];
