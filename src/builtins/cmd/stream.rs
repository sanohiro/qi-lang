//! コマンド実行 - stream

use super::helpers::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Arc;

/// stream-lines - コマンドのstdoutを行単位でストリームとして返す
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
/// 引数: コマンド（文字列 or [コマンド 引数...]）
/// 戻り値: Stream（各要素は行文字列）
/// 例: (cmd/stream-lines "tail -f /var/log/app.log")
///     (stream-take 10 s |> realize)
pub fn native_stream_lines(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["cmd/stream-lines", "1"],
        ));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let child = if cmd_args.is_empty() {
        // シェル経由（危険な文字が含まれる場合はエラー）
        check_shell_metacharacters(&cmd)?;

        #[cfg(unix)]
        let child = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdout(Stdio::piped())
            .spawn();

        #[cfg(windows)]
        let child = Command::new("cmd")
            .arg("/C")
            .arg(&cmd)
            .stdout(Stdio::piped())
            .spawn();

        child
    } else {
        // 直接実行
        Command::new(&cmd)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .spawn()
    };

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| fmt_msg(MsgKey::CmdExecutionFailed, &["Failed to capture stdout"]))?;

    let reader = Arc::new(RwLock::new(BufReader::new(stdout).lines()));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut lines = reader.write();
            match lines.next() {
                Some(Ok(line)) => Some(Value::String(line)),
                _ => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// stream-bytes - コマンドのstdoutをバイトチャンク単位でストリームとして返す
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
/// 引数: コマンド（文字列 or [コマンド 引数...]）, [オプション: チャンクサイズ（デフォルト4096）]
/// 戻り値: Stream（各要素はバイト配列の文字列表現）
/// 例: (cmd/stream-bytes "cat large-file.bin")
///     (cmd/stream-bytes "curl -L https://example.com/video.mp4" 8192)
pub fn native_stream_bytes(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["cmd/stream-bytes"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let chunk_size = if args.len() == 2 {
        match &args[1] {
            Value::Integer(n) if *n > 0 => *n as usize,
            _ => {
                return Err(fmt_msg(
                    MsgKey::MustBePositiveInteger,
                    &["cmd/stream-bytes", "chunk size"],
                ))
            }
        }
    } else {
        4096
    };

    let child = if cmd_args.is_empty() {
        // シェル経由（危険な文字が含まれる場合はエラー）
        check_shell_metacharacters(&cmd)?;

        #[cfg(unix)]
        let child = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdout(Stdio::piped())
            .spawn();

        #[cfg(windows)]
        let child = Command::new("cmd")
            .arg("/C")
            .arg(&cmd)
            .stdout(Stdio::piped())
            .spawn();

        child
    } else {
        // 直接実行
        Command::new(&cmd)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .spawn()
    };

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| fmt_msg(MsgKey::CmdExecutionFailed, &["Failed to capture stdout"]))?;

    let reader = Arc::new(RwLock::new(stdout));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut stdout = reader.write();
            let mut buffer = vec![0u8; chunk_size];
            match stdout.read(&mut buffer) {
                Ok(0) => None, // EOF
                Ok(n) => {
                    buffer.truncate(n);
                    // バイトデータを文字列として返す（Base64エンコード）
                    #[cfg(feature = "string-encoding")]
                    {
                        use base64::{engine::general_purpose, Engine as _};
                        Some(Value::String(general_purpose::STANDARD.encode(&buffer)))
                    }
                    #[cfg(not(feature = "string-encoding"))]
                    {
                        // fallback: デバッグ表現
                        Some(Value::String(format!("{:?}", buffer)))
                    }
                }
                Err(_) => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}
