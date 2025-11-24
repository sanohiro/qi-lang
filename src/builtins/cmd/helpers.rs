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
/// コマンドインジェクション攻撃に利用される可能性のある文字を検出する。
/// 検出された場合はエラーを返す。
pub(super) fn check_shell_metacharacters(cmd: &str) -> Result<(), String> {
    // 危険な文字の定義（文字列パターンと文字パターンの両方）
    const DANGEROUS_CHARS: &[&str] = &[
        ";", "|", "&", "$", "`", "(", ")", "<", ">", "\n", "\r", "\"", "'", "\\", "*", "?", "[",
        "]", "{", "}", "~", // グロブパターン、ブレース展開、チルダ展開
    ];

    // 制御文字とNULLバイトのチェック
    for ch in cmd.chars() {
        if ch.is_ascii_control() || ch == '\0' {
            return Err(fmt_msg(
                MsgKey::CmdDangerousCharacters,
                &[&format!("control character: {}", ch.escape_default())],
            ));
        }
    }

    // 危険な文字列パターンのチェック
    let found_chars: Vec<&str> = DANGEROUS_CHARS
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
            let args_slice: Vec<Value> = args_vec.iter().cloned().collect();
            let cmd = match &args_slice[0] {
                Value::String(s) => s.clone(),
                _ => return Err(fmt_msg(MsgKey::CmdFirstArgMustBeString, &[])),
            };
            let args: Result<Vec<String>, String> = args_slice[1..]
                .iter()
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
