//! Core状態管理・メタプログラミング関数
//!
//! 状態管理（4個）: atom, deref, swap!, reset!
//! メタ（6個）: eval, uvar, variable, macro?, macroexpand, source
//! 合計10個のCore関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::parser::Parser;
use crate::value::Value;
use im::Vector;
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
            !matches!(&**s, "nil" | "true" | "false")
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
        // データ構造の場合はValue→Exprに直接変換（Lispの「データ→コード」文化に沿う）
        value => {
            let expr = evaluator.value_to_expr(value)?;
            evaluator.eval(&expr)
        }
    }
}

/// macroexpand - マクロを展開
///
/// Qiのマクロ（defn, defn-等）を展開した形を返す
///
/// # 例
/// ```qi
/// (macroexpand '(defn add [a b] (+ a b)))
/// ;=> (def add (fn [a b] (+ a b)))
///
/// (macroexpand '(defn greet "Greeting function" [name] (str "Hello, " name)))
/// ;=> (do (def __doc__greet "Greeting function") (def greet (fn [name] (str "Hello, " name))))
/// ```
pub fn native_macroexpand(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["macroexpand"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) if !items.is_empty() => {
            // 最初の要素がシンボルかチェック
            if let Some(Value::Symbol(sym)) = items.get(0) {
                match &**sym {
                    "defn" => expand_defn(items, false),
                    "defn-" => expand_defn(items, true),
                    _ => Ok(args[0].clone()), // 展開不要
                }
            } else {
                Ok(args[0].clone())
            }
        }
        _ => Ok(args[0].clone()), // マクロ形式でない
    }
}

/// source - シンボルの定義元を表示
///
/// # 例
/// ```qi
/// (defn add [a b] (+ a b))
/// (source 'add)
/// ;=> "User-defined function: add"
/// ```
pub fn native_source(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["source"]));
    }

    let symbol = match &args[0] {
        Value::Symbol(s) => s.as_ref(),
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["source", "a symbol"])),
    };

    // グローバル環境から値を取得
    // Evaluatorの内部フィールドに直接アクセスできないため、
    // 存在チェックしてから取得
    let var_check_code = format!("(variable '{})", symbol);
    let mut parser = Parser::new(&var_check_code).map_err(|e| format!("source: {}", e))?;
    let expr = parser.parse().map_err(|e| format!("source: {}", e))?;
    let is_var = evaluator.eval(&expr)?;

    if !is_var.is_truthy() {
        return Err(fmt_msg(MsgKey::UndefinedVar, &[symbol]));
    }

    // 値を取得
    let get_code = symbol.to_string();
    let mut parser = Parser::new(&get_code).map_err(|e| format!("source: {}", e))?;
    let expr = parser.parse().map_err(|e| format!("source: {}", e))?;
    let value = evaluator.eval(&expr).ok();

    match value {
        Some(Value::Function(_)) => {
            // ユーザー定義関数
            Ok(Value::String(format!(
                "User-defined function: {}
Location: <source information not available>

Note: Source code tracking is not yet fully implemented.
You can view the function by examining the original file.",
                symbol
            )))
        }
        Some(Value::NativeFunc(_)) => {
            // ネイティブ関数（Rustで実装）
            Ok(Value::String(format!(
                "Native function: {}
Implemented in: src/builtins/

This is a built-in function implemented in Rust.
Use :doc {} to see documentation.",
                symbol, symbol
            )))
        }
        Some(Value::Macro(_)) => {
            // マクロ
            Ok(Value::String(format!("Macro: {}", symbol)))
        }
        Some(_) => {
            // その他の値
            Ok(Value::String(format!(
                "Value binding: {}
Type: {}",
                symbol,
                value.unwrap().type_name()
            )))
        }
        None => Err(fmt_msg(MsgKey::UndefinedVar, &[symbol])),
    }
}

/// defn/defn- を展開
fn expand_defn(items: &Vector<Value>, _is_private: bool) -> Result<Value, String> {
    // (defn name [params] body...) -> (def name (fn [params] body...))
    // (defn name doc [params] body...) -> (do (def __doc__name doc) (def name (fn [params] body...)))

    if items.len() < 4 {
        return Err("defn: requires at least name, parameters, and body".to_string());
    }

    let name = items
        .get(1)
        .ok_or_else(|| "defn: missing name".to_string())?
        .clone();

    // ドキュメント文字列があるかチェック
    if let Some(Value::String(doc)) = items.get(2) {
        // ドキュメント付き: (defn name doc [params] body...)
        // -> (do (def __doc__name doc) (def name (fn [params] body...)))

        if items.len() < 5 {
            return Err("defn: requires parameters and body after docstring".to_string());
        }

        let params = items
            .get(3)
            .ok_or_else(|| "defn: missing parameters".to_string())?;
        let body: Vector<Value> = items.iter().skip(4).cloned().collect();

        // __doc__name シンボルを生成
        let doc_name = if let Value::Symbol(s) = &name {
            Value::Symbol(format!("__doc__{}", s).into())
        } else {
            return Err("defn: name must be a symbol".to_string());
        };

        // (def __doc__name doc)
        let def_doc = Value::List(
            vec![
                Value::Symbol("def".into()),
                doc_name,
                Value::String(doc.clone()),
            ]
            .into_iter()
            .collect(),
        );

        // (fn [params] body...)
        let mut fn_form = vec![Value::Symbol("fn".into()), params.clone()];
        fn_form.extend(body);
        let fn_value = Value::List(fn_form.into_iter().collect());

        // (def name (fn [params] body...))
        let def_name = Value::List(
            vec![Value::Symbol("def".into()), name, fn_value]
                .into_iter()
                .collect(),
        );

        // (do (def __doc__name doc) (def name (fn [params] body...)))
        Ok(Value::List(
            vec![Value::Symbol("do".into()), def_doc, def_name]
                .into_iter()
                .collect(),
        ))
    } else {
        // ドキュメントなし: (defn name [params] body...)
        // -> (def name (fn [params] body...))

        let params = items
            .get(2)
            .ok_or_else(|| "defn: missing parameters".to_string())?;
        let body: Vector<Value> = items.iter().skip(3).cloned().collect();

        // (fn [params] body...)
        let mut fn_form = vec![Value::Symbol("fn".into()), params.clone()];
        fn_form.extend(body);
        let fn_value = Value::List(fn_form.into_iter().collect());

        // (def name (fn [params] body...))
        Ok(Value::List(
            vec![Value::Symbol("def".into()), name, fn_value]
                .into_iter()
                .collect(),
        ))
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category core/state-meta
/// @qi-doc:functions atom, deref, swap!, reset!, eval, uvar, variable, macro?, macroexpand
///
/// 注意: swap!, evalはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("atom", native_atom),
    ("deref", native_deref),
    ("reset!", native_reset),
    ("uvar", native_uvar),
    ("variable", native_variable),
    ("macro?", native_macro_q),
    ("macroexpand", native_macroexpand),
];
