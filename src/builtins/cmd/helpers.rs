//! コマンド実行 - ヘルパー関数

use super::types::ProcessStreams;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;

/// グローバルプロセスマップ（PID -> ストリーム）
pub(super) static PROCESS_MAP: Lazy<Mutex<HashMap<u32, ProcessStreams>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// シェルメタ文字をチェック
///
/// **セキュリティ警告**: この関数は最小限のチェックのみを行います。
/// ユーザー入力を含むコマンドを実行する場合は、**必ずベクター形式**を使用してください。
///
/// 文字列形式でコマンドを実行する場合、シェル経由で実行されるため、
/// コマンドインジェクション攻撃のリスクがあります。
///
/// # チェック内容
/// - NULLバイト（明らかな攻撃）
/// - 制御文字（改行、復帰、タブ以外）
/// - コマンド連結文字（`;`, `|`, `&`）
///
/// # 安全な使い方
/// ```qi
/// ; ユーザー入力を含む場合は必ずベクター形式を使用
/// (cmd/exec ["ls" "-la" user-input])
///
/// ; ハードコードされたコマンドのみ文字列形式を使用
/// (cmd/exec "ls -la")
/// ```
pub(super) fn check_shell_metacharacters(cmd: &str) -> Result<(), String> {
    // NULLバイトのチェック（明らかな攻撃）
    if cmd.contains('\0') {
        return Err(fmt_msg(
            MsgKey::CmdDangerousCharacters,
            &["NULL byte"],
        ));
    }

    // 制御文字のチェック（改行、復帰、タブは許可）
    for ch in cmd.chars() {
        if ch.is_ascii_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            return Err(fmt_msg(
                MsgKey::CmdDangerousCharacters,
                &[&format!("control character: {}", ch.escape_default())],
            ));
        }
    }

    // コマンド連結文字のチェック（明らかな攻撃パターン）
    const DANGEROUS_PATTERNS: &[&str] = &[
        ";", // コマンド連結
        "|", // パイプ（コマンド連結）
        "&", // バックグラウンド実行・AND連結
    ];

    let found_chars: Vec<&str> = DANGEROUS_PATTERNS
        .iter()
        .filter(|&&c| cmd.contains(c))
        .copied()
        .collect();

    if !found_chars.is_empty() {
        return Err(fmt_msg(
            MsgKey::CmdDangerousCharacters,
            &[&found_chars.join(", ")],
        ));
    }

    Ok(())
}

/// コマンド実行結果をMapに変換
pub(super) fn result_to_map(stdout: Vec<u8>, stderr: Vec<u8>, exit_code: i32) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "stdout".to_string(),
        Value::String(String::from_utf8_lossy(&stdout).to_string()),
    );
    map.insert(
        "stderr".to_string(),
        Value::String(String::from_utf8_lossy(&stderr).to_string()),
    );
    map.insert("exit".to_string(), Value::Integer(exit_code as i64));
    Value::Map(convert_string_map_to_mapkey(map))
}

/// コマンド引数を解析（文字列 or ベクタ）
pub(super) fn parse_command_args(val: &Value) -> Result<(String, Vec<String>), String> {
    match val {
        Value::String(cmd) => {
            // シェル経由で実行
            Ok((cmd.clone(), vec![]))
        }
        Value::List(args_vec) | Value::Vector(args_vec) => {
            if args_vec.is_empty() {
                return Err(fmt_msg(MsgKey::CmdEmptyCommand, &[]));
            }
            // イテレータを直接使用（不要なクローンを削減）
            let mut iter = args_vec.iter();
            let cmd = match iter.next() {
                Some(Value::String(s)) => s.clone(),
                Some(_) => return Err(fmt_msg(MsgKey::CmdFirstArgMustBeString, &[])),
                None => return Err(fmt_msg(MsgKey::CmdEmptyCommand, &[])),
            };
            let args: Result<Vec<String>, String> = iter
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Err(fmt_msg(MsgKey::CmdArgsMustBeStrings, &[])),
                })
                .collect();
            Ok((cmd, args?))
        }
        _ => Err(fmt_msg(MsgKey::CmdInvalidArgument, &[])),
    }
}
