//! コマンド実行 - interactive

use super::helpers::*;
use crate::builtins::util::convert_string_map_to_mapkey;
use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;

/// interactive - 双方向インタラクティブプロセスを起動
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
/// 戻り値: プロセスハンドル（Map形式）
/// 例: (def py (cmd/interactive "python3 -i"))
///     (cmd/write py "print(1+1)\n")
///     (cmd/read-line py)
pub fn native_interactive(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "cmd/interactive");

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let child = if cmd_args.is_empty() {
        // シェル経由（危険な文字が含まれる場合はエラー）
        check_shell_metacharacters(&cmd)?;

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

    Ok(Value::Map(convert_string_map_to_mapkey(handle)))
}

/// write - プロセスのstdinに書き込む
/// 引数: プロセスハンドル, データ（文字列）
/// 戻り値: nil
/// 例: (cmd/write py "print(1+1)\n")
pub fn native_proc_write(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "cmd/write");

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/write", "a process handle"],
            ))
        }
    };

    let pid = match handle.get(&crate::value::MapKey::String("pid".to_string())) {
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
    check_args!(args, 1, "cmd/read-line");

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/read-line", "a process handle"],
            ))
        }
    };

    let pid = match handle.get(&crate::value::MapKey::String("pid".to_string())) {
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
    check_args!(args, 1, "cmd/wait");

    let handle = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["cmd/wait", "a process handle"],
            ))
        }
    };

    let pid = match handle.get(&crate::value::MapKey::String("pid".to_string())) {
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

    Ok(Value::Map(convert_string_map_to_mapkey(result)))
}
