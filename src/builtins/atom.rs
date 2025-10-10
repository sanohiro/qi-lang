//! 状態管理関数（Atom）

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::RwLock;
use std::sync::Arc;

/// atom - アトムを作成
///
/// 引数:
/// - value: 初期値
///
/// 戻り値:
/// - アトム
///
/// 例:
/// ```lisp
/// (def counter (atom 0))
/// ```
pub fn native_atom(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["atom"]));
    }

    Ok(Value::Atom(Arc::new(RwLock::new(args[0].clone()))))
}

/// deref - アトムから値を取得
///
/// 引数:
/// - atom: アトム
///
/// 戻り値:
/// - アトムの現在の値
///
/// 例:
/// ```lisp
/// (def counter (atom 0))
/// (deref counter)  ;; => 0
/// ```
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
///
/// 引数:
/// - atom: アトム
/// - value: 新しい値
///
/// 戻り値:
/// - 新しい値
///
/// 例:
/// ```lisp
/// (def counter (atom 0))
/// (reset! counter 10)  ;; => 10
/// (deref counter)      ;; => 10
/// ```
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
///
/// 引数:
/// - atom: アトム
/// - f: 更新関数
/// - args...: 関数に渡す追加の引数
///
/// 戻り値:
/// - 新しい値
///
/// 例:
/// ```lisp
/// (def counter (atom 0))
/// (swap! counter inc)      ;; => 1
/// (swap! counter + 5)      ;; => 6
/// ```
pub fn native_swap(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
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
