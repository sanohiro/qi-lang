//! デバッグ用組み込み関数
//!
//! トレース、ブレークポイント、スタックトレースなどのデバッグ機能

use crate::check_args;
use crate::builtins::util::kw;
use crate::debugger::GLOBAL_DEBUGGER;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::LazyLock;

/// トレース対象の関数名セット（グローバル）
pub static TRACED_FUNCTIONS: LazyLock<RwLock<HashSet<String>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

/// debug/trace - トレース機能の有効/無効を切り替え
///
/// 引数:
/// - enabled: boolean (true=有効, false=無効)
///
/// 戻り値: nil
///
/// @qi-doc:category debug
/// @qi-doc:description トレース機能を有効/無効にする
pub fn native_debug_trace(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "debug/trace");

    let enabled = match &args[0] {
        Value::Bool(b) => *b,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["debug/trace", "a boolean"],
            ))
        }
    };

    // デバッガが未初期化の場合は初期化
    {
        let mut guard = GLOBAL_DEBUGGER.write();
        if guard.is_none() {
            *guard = Some(crate::debugger::Debugger::new(true));
        }
    }

    // トレースの有効/無効を切り替え
    if let Some(ref mut dbg) = *GLOBAL_DEBUGGER.write() {
        if enabled {
            dbg.enable_trace();
        } else {
            dbg.disable_trace();
        }
    }

    Ok(Value::Nil)
}

/// debug/break - 実行を一時停止（ブレークポイント）
///
/// 引数: なし
///
/// 戻り値: nil
///
/// @qi-doc:category debug
/// @qi-doc:description 実行を一時停止する（デバッガがアタッチされている場合）
pub fn native_debug_break(_args: &[Value]) -> Result<Value, String> {
    if let Some(ref mut dbg) = *GLOBAL_DEBUGGER.write() {
        dbg.pause();
        eprintln!("[DEBUG] Breakpoint hit. Execution paused.");
    }

    Ok(Value::Nil)
}

/// debug/stack - 現在のコールスタックを表示
///
/// 引数: なし
///
/// 戻り値: スタックトレース文字列
///
/// @qi-doc:category debug
/// @qi-doc:description 現在のコールスタックを返す
pub fn native_debug_stack(_args: &[Value]) -> Result<Value, String> {
    let trace = if let Some(ref dbg) = *GLOBAL_DEBUGGER.read() {
        dbg.stack_trace()
    } else {
        "Debugger not enabled\n".to_string()
    };

    Ok(Value::String(trace))
}

/// debug/info - デバッガの現在の状態を表示
///
/// 引数: なし
///
/// 戻り値: デバッグ情報のマップ
///
/// @qi-doc:category debug
/// @qi-doc:description デバッガの現在の状態を返す
pub fn native_debug_info(_args: &[Value]) -> Result<Value, String> {
    let mut info = crate::new_hashmap();

    if let Some(ref dbg) = *GLOBAL_DEBUGGER.read() {
        info.insert(kw("enabled"), Value::Bool(dbg.is_enabled()));
        info.insert(kw("state"), Value::String(format!("{:?}", dbg.state())));
        info.insert(
            kw("stack-depth"),
            Value::Integer(dbg.call_stack().len() as i64),
        );
    } else {
        info.insert(kw("enabled"), Value::Bool(false));
    }

    Ok(Value::Map(info))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category debug
/// @qi-doc:functions trace, break, stack, info
pub const FUNCTIONS: super::NativeFunctions = &[
    ("debug/trace", native_debug_trace),
    ("debug/break", native_debug_break),
    ("debug/stack", native_debug_stack),
    ("debug/info", native_debug_info),
];
