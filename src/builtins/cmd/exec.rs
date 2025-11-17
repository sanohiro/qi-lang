//! コマンド実行 - exec

use super::helpers::*;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::process::Command;

/// exec - コマンド実行（終了コードを返す）
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
/// 戻り値: 終了コード（整数）
/// 例: (cmd/exec "ls -la")  ;=> 0
///     (cmd/exec ["ls" "-la"])  ;=> 0
///     (cmd/exec "false")  ;=> 1
pub fn native_exec(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/exec", "1"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let output = if cmd_args.is_empty() {
        // シェル経由（セキュリティ警告を表示）
        check_shell_metacharacters(&cmd);

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
            Ok(Value::Integer(exit_code as i64))
        }
        Err(e) => Err(fmt_msg(MsgKey::CmdExecutionFailed, &[&e.to_string()])),
    }
}

/// exec! - コマンド実行（詳細版）
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
/// 戻り値: {:stdout "..." :stderr "..." :exit 0}
/// エラー時: Err(エラーメッセージ)
/// 例: (cmd/exec! "ls -la")  ;=> {:stdout "..." :stderr "" :exit 0}
///     (cmd/exec! ["ls" "-la"])  ;=> {:stdout "..." :stderr "" :exit 0}
pub fn native_exec_bang(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["cmd/exec!", "1"]));
    }

    let (cmd, cmd_args) = parse_command_args(&args[0])?;

    let output = if cmd_args.is_empty() {
        // シェル経由（セキュリティ警告を表示）
        check_shell_metacharacters(&cmd);

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
