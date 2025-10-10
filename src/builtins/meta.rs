//! メタプログラミング関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::parser::Parser;
use crate::value::Value;
use std::sync::atomic::{AtomicU64, Ordering};

/// グローバルなuvarカウンター
static UVAR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// uvar - 一意な変数を生成
///
/// 引数: なし
///
/// 戻り値: 一意な変数
///
/// 例:
/// ```lisp
/// (uvar)  ;; => #<uvar:0>
/// (uvar)  ;; => #<uvar:1>
/// ```
pub fn native_uvar(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["uvar"]));
    }

    let id = UVAR_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Uvar(id))
}

/// variable - 変数かどうかをチェック
///
/// 引数:
/// - value: チェックする値
///
/// 戻り値: 変数ならtrue、そうでなければfalse
///
/// 例:
/// ```lisp
/// (variable 'x)        ;; => true
/// (variable #<uvar:0>) ;; => true
/// (variable 42)        ;; => false
/// ```
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
///
/// 引数:
/// - value: チェックする値
///
/// 戻り値: マクロならtrue、そうでなければfalse
///
/// 例:
/// ```lisp
/// (macro? my-macro)  ;; => true
/// (macro? +)         ;; => false
/// ```
pub fn native_macro_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["macro?"]));
    }

    Ok(Value::Bool(matches!(&args[0], Value::Macro(_))))
}

/// eval - 式を評価
///
/// 引数:
/// - expr: 評価する式（文字列またはデータ構造）
///
/// 戻り値: 評価結果
///
/// 例:
/// ```lisp
/// (eval "(+ 1 2)")     ;; => 3
/// (eval '(+ 1 2))      ;; => 3
/// ```
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
