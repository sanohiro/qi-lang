//! コマンド実行関数
//!
//! Unixコマンドをパイプラインで実行できる関数群。
//! データの流れとしてコマンドを扱い、|> と統合される。
//!
//! このモジュールは `cmd-exec` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;

/// プロセスストリーム（双方向通信用）
type ProcessStreams = (
    Option<std::process::ChildStdin>,
    Option<BufReader<std::process::ChildStdout>>,
    Option<BufReader<std::process::ChildStderr>>,
    std::process::Child,
);

/// グローバルプロセスマップ（PID -> ストリーム）
static PROCESS_MAP: Lazy<Mutex<HashMap<u32, ProcessStreams>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

/// stream-lines - コマンドのstdoutを行単位でストリームとして返す
/// 引数: コマンド（文字列 or [コマンド 引数...]）
/// 戻り値: Stream（各要素は行文字列）
/// 例: (cmd/stream-lines "tail -f /var/log/app.log")
///     (stream-take 10 s |> realize)
pub fn native_stream_lines(args: &[Value]) -> Result<Value, String> {
    use crate::value::Stream;
    use parking_lot::RwLock;
    use std::io::{BufRead, BufReader};
    use std::sync::Arc;

    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["cmd/stream-lines", "1"],
        ));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let child = if cmd_args.is_empty() {
        // シェル経由
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
/// 引数: コマンド（文字列 or [コマンド 引数...]）, [オプション: チャンクサイズ（デフォルト4096）]
/// 戻り値: Stream（各要素はバイト配列の文字列表現）
/// 例: (cmd/stream-bytes "cat large-file.bin")
///     (cmd/stream-bytes "curl -L https://example.com/video.mp4" 8192)
pub fn native_stream_bytes(args: &[Value]) -> Result<Value, String> {
    use crate::value::Stream;
    use parking_lot::RwLock;
    use std::io::Read;
    use std::sync::Arc;

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
        // シェル経由
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

/// interactive - 双方向インタラクティブプロセスを起動
/// 引数: コマンド（文字列 or [コマンド 引数...]）
/// 戻り値: プロセスハンドル（Map形式）
/// 例: (def py (cmd/interactive "python3 -i"))
///     (cmd/write py "print(1+1)\n")
///     (cmd/read-line py)
pub fn native_interactive(args: &[Value]) -> Result<Value, String> {
    use std::io::BufReader;

    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/interactive", "1"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

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

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    };

    let pid = child.id();
    let stdin = child.stdin.take();
    let stdout = child.stdout.take().map(BufReader::new);
    let stderr = child.stderr.take().map(BufReader::new);

    // プロセスハンドルをMapとして返す
    let mut handle = HashMap::new();
    handle.insert("pid".to_string(), Value::Integer(pid as i64));
    handle.insert(
        "stdin".to_string(),
        Value::Atom(Arc::new(RwLock::new(Value::String(format!(
            "#<stdin:{}>",
            pid
        ))))),
    );
    handle.insert(
        "stdout".to_string(),
        Value::Atom(Arc::new(RwLock::new(Value::String(format!(
            "#<stdout:{}>",
            pid
        ))))),
    );
    handle.insert(
        "stderr".to_string(),
        Value::Atom(Arc::new(RwLock::new(Value::String(format!(
            "#<stderr:{}>",
            pid
        ))))),
    );

    // グローバルプロセスマップに登録
    PROCESS_MAP
        .lock()
        .insert(pid, (stdin, stdout, stderr, child));

    Ok(Value::Map(handle))
}

/// write - プロセスのstdinに書き込む
/// 引数: プロセスハンドル, データ（文字列）
/// 戻り値: nil
/// 例: (cmd/write py "print(1+1)\n")
pub fn native_proc_write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["cmd/write"]));
    }

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/write", "a process handle"],
            ))
        }
    };

    let pid = match handle.get("pid") {
        Some(Value::Integer(n)) => *n as u32,
        _ => return Err(fmt_msg(MsgKey::CmdInvalidProcessHandle, &[])),
    };

    let data = match &args[1] {
        Value::String(s) => s.as_bytes(),
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["cmd/write", "a string"])),
    };

    let mut map = PROCESS_MAP.lock();
    let streams = map
        .get_mut(&pid)
        .ok_or_else(|| fmt_msg(MsgKey::CmdProcessNotFound, &[&pid.to_string()]))?;

    if let Some(stdin) = &mut streams.0 {
        stdin
            .write_all(data)
            .map_err(|e| fmt_msg(MsgKey::CmdWriteFailed, &[&e.to_string()]))?;
        stdin
            .flush()
            .map_err(|e| fmt_msg(MsgKey::CmdWriteFailed, &[&e.to_string()]))?;
    } else {
        return Err(fmt_msg(MsgKey::CmdStdinClosed, &[]));
    }

    Ok(Value::Nil)
}

/// read-line - プロセスのstdoutから1行読み取る
/// 引数: プロセスハンドル
/// 戻り値: 読み取った行（文字列）、EOFならnil
/// 例: (cmd/read-line py)
pub fn native_proc_read_line(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["cmd/read-line"]));
    }

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/read-line", "a process handle"],
            ))
        }
    };

    let pid = match handle.get("pid") {
        Some(Value::Integer(n)) => *n as u32,
        _ => return Err(fmt_msg(MsgKey::CmdInvalidProcessHandle, &[])),
    };

    let mut map = PROCESS_MAP.lock();
    let streams = map
        .get_mut(&pid)
        .ok_or_else(|| fmt_msg(MsgKey::CmdProcessNotFound, &[&pid.to_string()]))?;

    if let Some(stdout) = &mut streams.1 {
        let mut line = String::new();
        match stdout.read_line(&mut line) {
            Ok(0) => Ok(Value::Nil), // EOF
            Ok(_) => {
                // 末尾の改行を削除
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Value::String(line))
            }
            Err(e) => Err(fmt_msg(MsgKey::CmdReadFailed, &[&e.to_string()])),
        }
    } else {
        Err(fmt_msg(MsgKey::CmdStdoutClosed, &[]))
    }
}

/// wait - プロセスの終了を待つ
/// 引数: プロセスハンドル
/// 戻り値: {:exit exit_code :stderr "..."}
/// 例: (cmd/wait py)
pub fn native_proc_wait(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["cmd/wait"]));
    }

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/wait", "a process handle"],
            ))
        }
    };

    let pid = match handle.get("pid") {
        Some(Value::Integer(n)) => *n as u32,
        _ => return Err(fmt_msg(MsgKey::CmdInvalidProcessHandle, &[])),
    };

    let mut map = PROCESS_MAP.lock();
    let mut streams = map
        .remove(&pid)
        .ok_or_else(|| fmt_msg(MsgKey::CmdProcessNotFound, &[&pid.to_string()]))?;

    // stdinを閉じる（EOFを送信）
    drop(streams.0);

    // stderrを読み取る
    let mut stderr_content = String::new();
    if let Some(mut stderr) = streams.2 {
        let _ = stderr.read_to_string(&mut stderr_content);
    }

    // プロセスの終了を待つ
    let status = streams
        .3
        .wait()
        .map_err(|e| fmt_msg(MsgKey::CmdWaitFailed, &[&e.to_string()]))?;

    let exit_code = status.code().unwrap_or(-1);

    let mut result = HashMap::new();
    result.insert("exit".to_string(), Value::Integer(exit_code as i64));
    result.insert("stderr".to_string(), Value::String(stderr_content));

    Ok(Value::Map(result))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
pub const FUNCTIONS: &[(&str, fn(&[Value]) -> Result<Value, String>)] = &[
    ("cmd/exec", native_exec),
    ("cmd/sh", native_sh),
    ("cmd/pipe", native_pipe),
    ("cmd/lines", native_lines),
    ("cmd/stream-lines", native_stream_lines),
    ("cmd/stream-bytes", native_stream_bytes),
    ("cmd/interactive", native_interactive),
    ("cmd/write", native_proc_write),
    ("cmd/read-line", native_proc_read_line),
    ("cmd/wait", native_proc_wait),
];
