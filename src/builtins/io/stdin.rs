use super::*;

/// stdin-read-line - 標準入力から1行を読み込む
///
/// 標準入力から1行を読み込みます。改行文字は自動的に削除されます。
/// EOF（ファイル終了）に達した場合はnilを返します。
///
/// # 引数
/// なし
///
/// # 戻り値
/// `string | nil` - 読み込んだ行、またはEOFの場合はnil
///
/// # 使用例
/// ```qi
/// (io/stdin-read-line)
/// (io/read-prompt "Enter name: " |> io/stdin-read-line)
/// ```
pub fn native_stdin_read_line(_args: &[Value]) -> Result<Value, String> {
    // DAPモードの場合、標準入力待ちの通知を出力
    #[cfg(feature = "dap-server")]
    {
        let is_dap_mode = crate::debugger::GLOBAL_DEBUGGER.read().is_some();
        if is_dap_mode {
            use crate::i18n::{ui_msg, UiMsg};
            eprintln!("{}", ui_msg(UiMsg::DapStdinWaiting));
            eprintln!("{}", ui_msg(UiMsg::DapStdinInstructions));
        }
    }

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    match handle.read_line(&mut line) {
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
        Err(e) => Err(fmt_msg(
            MsgKey::IoReadLinesFailedToRead,
            &["stdin", &e.to_string()],
        )),
    }
}

/// stdin-lines - 標準入力から全行を読み込む
/// 引数: なし
/// 戻り値: 行の配列（Vector）
///
/// 使用例:
/// ```qi
/// (io/stdin-lines
///  |> (map str/trim)
///  |> (filter (fn [s] (not (str/empty? s))))
///  |> (each println))
/// ```
pub fn native_stdin_read_lines(_args: &[Value]) -> Result<Value, String> {
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    let lines: Vec<Value> = reader
        .lines()
        .map_while(Result::ok)
        .map(Value::String)
        .collect();

    Ok(Value::Vector(lines.into()))
}

// ========================================
// 関数登録テーブル
// ========================================
