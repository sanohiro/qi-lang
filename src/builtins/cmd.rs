//! コマンド実行関数
//!
//! Unixコマンドをパイプラインで実行できる関数群。
//! データの流れとしてコマンドを扱い、|> と統合される。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

/// コマンド実行結果をMapに変換
fn result_to_map(stdout: Vec<u8>, stderr: Vec<u8>, exit_code: i32) -> Value {
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
    Value::Map(map)
}

/// コマンド引数を解析（文字列 or ベクタ）
fn parse_command_args(val: &Value) -> Result<(String, Vec<String>), String> {
    match val {
        Value::String(cmd) => {
            // シェル経由で実行
            Ok((cmd.clone(), vec![]))
        }
        Value::List(args) => {
            if args.is_empty() {
                return Err(fmt_msg(MsgKey::CmdEmptyCommand, &[]));
            }
            let cmd = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err(fmt_msg(MsgKey::CmdFirstArgMustBeString, &[])),
            };
            let args: Result<Vec<String>, String> = args[1..]
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

/// exec - コマンド実行（結果を返す）
/// 引数: コマンド（文字列 or [コマンド 引数...]）
/// 戻り値: {:stdout "..." :stderr "..." :exit 0}
/// 例: (cmd/exec "ls -la")
///     (cmd/exec ["ls" "-la"])
pub fn native_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/exec", "1"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let output = if cmd_args.is_empty() {
        // シェル経由
        #[cfg(unix)]
        let result = Command::new("sh").arg("-c").arg(&cmd).output();

        #[cfg(windows)]
        let result = Command::new("cmd").arg("/C").arg(&cmd).output();

        result
    } else {
        // 直接実行
        Command::new(&cmd).args(&cmd_args).output()
    };

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(result_to_map(output.stdout, output.stderr, exit_code))
        }
        Err(e) => Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    }
}

/// sh - シェル経由で実行
/// 引数: コマンド文字列
/// 戻り値: {:stdout "..." :stderr "..." :exit 0}
/// 例: (cmd/sh "cat *.txt | grep pattern | wc -l")
pub fn native_sh(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/sh", "1"]));
    }

    let cmd = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["cmd/sh", "a string"])),
    };

    #[cfg(unix)]
    let output = Command::new("sh").arg("-c").arg(cmd).output();

    #[cfg(windows)]
    let output = Command::new("cmd").arg("/C").arg(cmd).output();

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(result_to_map(output.stdout, output.stderr, exit_code))
        }
        Err(e) => Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    }
}

/// pipe - データをコマンドの標準入力へパイプ
/// 引数: コマンド, 入力データ（文字列 or リスト）
/// 戻り値: {:stdout "..." :stderr "..." :exit 0}
/// 例: ("hello\nworld\n" |> (cmd/pipe "sort"))
///     (["line1" "line2"] |> (cmd/pipe "wc -l"))
pub fn native_pipe(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/pipe", "2"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    // 入力データを準備
    let input_data = match &args[1] {
        Value::String(s) => s.as_bytes().to_vec(),
        Value::List(items) | Value::Vector(items) => {
            // リスト/ベクタの場合、各要素を改行で結合
            let lines: Vec<String> = items
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    _ => format!("{:?}", v),
                })
                .collect();
            lines.join("\n").into_bytes()
        }
        _ => {
            // その他の型は文字列化
            format!("{:?}", args[1]).into_bytes()
        }
    };

    let child = if cmd_args.is_empty() {
        // シェル経由
        #[cfg(unix)]
        let child = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        #[cfg(windows)]
        let child = Command::new("cmd")
            .arg("/C")
            .arg(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        child
    } else {
        // 直接実行
        Command::new(&cmd)
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };

    match child {
        Ok(mut child) => {
            // 標準入力に書き込む
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(e) = stdin.write_all(&input_data) {
                    return Err(fmt_msg(MsgKey::CmdWriteFailed, &[&e.to_string()]));
                }
                drop(stdin); // EOF送信
            }

            // 出力を取得
            match child.wait_with_output() {
                Ok(output) => {
                    let exit_code = output.status.code().unwrap_or(-1);
                    Ok(result_to_map(output.stdout, output.stderr, exit_code))
                }
                Err(e) => Err(fmt_msg(MsgKey::CmdWaitFailed, &[&e.to_string()])),
            }
        }
        Err(e) => Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    }
}

/// lines - テキストを行のリストに分割（ヘルパー）
/// 引数: テキスト
/// 戻り値: 行のリスト
/// 例: ("a\nb\nc" |> cmd/lines) => ["a" "b" "c"]
pub fn native_lines(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/lines", "1"]));
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["cmd/lines", "a string"])),
    };

    let lines: Vec<Value> = text
        .lines()
        .map(|line| Value::String(line.to_string()))
        .collect();

    Ok(Value::List(lines))
}
