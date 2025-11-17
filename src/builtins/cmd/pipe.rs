//! コマンド実行 - pipe

use super::helpers::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::io::Write;
use std::process::{Command, Stdio};

/// pipe - コマンドを実行して標準出力を返す
///
/// ## セキュリティ警告
///
/// 文字列としてコマンドを渡す場合、シェル経由で実行されるため、
/// コマンドインジェクション攻撃のリスクがあります。
///
/// **安全な使い方**:
/// - ハードコードされたコマンド文字列のみ使用
/// - ユーザー入力を含む場合は配列形式で渡す（`["cmd" "arg1" "arg2"]`）
///
/// 引数: コマンド, [入力データ（文字列 or リスト）]
/// 戻り値: 標準出力（文字列）。コマンドが失敗した場合はエラーを投げる
/// 例: (cmd/pipe "ls -la")  ;=> "total 48\ndrwxr-xr-x ...\n"
///     ("hello\nworld\n" |> (cmd/pipe "sort"))  ;=> "hello\nworld\n"
///     (["line1" "line2"] |> (cmd/pipe "wc -l"))  ;=> "       2\n"
pub fn native_pipe(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/pipe", "1 or 2"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    // 入力データを準備（第2引数がある場合）
    let input_data = if args.len() == 2 {
        match &args[1] {
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
        }
    } else {
        // 第2引数がない場合は空の入力
        Vec::new()
    };

    let child = if cmd_args.is_empty() {
        // シェル経由（セキュリティ警告を表示）
        check_shell_metacharacters(&cmd);

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
                    if exit_code == 0 {
                        // 成功: stdoutを返す
                        Ok(Value::String(
                            String::from_utf8_lossy(&output.stdout).to_string(),
                        ))
                    } else {
                        // 失敗: エラーを投げる（stderrも含める）
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(format!(
                            "Command failed with exit code {}: {}{}",
                            exit_code,
                            if stderr.is_empty() { "" } else { "\n" },
                            stderr
                        ))
                    }
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

    Ok(Value::List(lines.into()))
}

/// pipe! - コマンドを実行して詳細情報を返す
///
/// ## セキュリティ警告
///
/// 文字列としてコマンドを渡す場合、シェル経由で実行されるため、
/// コマンドインジェクション攻撃のリスクがあります。
///
/// **安全な使い方**:
/// - ハードコードされたコマンド文字列のみ使用
/// - ユーザー入力を含む場合は配列形式で渡す（`["cmd" "arg1" "arg2"]`）
///
/// 引数: コマンド, [入力データ（文字列 or リスト）]
/// 戻り値: [stdout stderr exitcode] ベクタ
/// 例: (cmd/pipe! "ls -la")  ;=> ["total 48\n..." "" 0]
///     (let [[out err code] ("test" |> (cmd/pipe! "cat"))] ...)
///     (["line1" "line2"] |> (cmd/pipe! ["wc" "-l"]))  ;=> ["       2\n" "" 0]
pub fn native_pipe_bang(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/pipe!", "1 or 2"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    // 入力データを準備（第2引数がある場合）
    let input_data = if args.len() == 2 {
        match &args[1] {
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
        }
    } else {
        // 第2引数がない場合は空の入力
        Vec::new()
    };

    let child = if cmd_args.is_empty() {
        // シェル経由（セキュリティ警告を表示）
        check_shell_metacharacters(&cmd);

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
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                    // [stdout stderr exitcode] のベクタを返す
                    Ok(Value::Vector(
                        vec![
                            Value::String(stdout),
                            Value::String(stderr),
                            Value::Integer(exit_code as i64),
                        ]
                        .into(),
                    ))
                }
                Err(e) => Err(fmt_msg(MsgKey::CmdWaitFailed, &[&e.to_string()])),
            }
        }
        Err(e) => Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    }
}
