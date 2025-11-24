//! コマンドライン引数パース関数

use crate::builtins::util::convert_string_map_to_mapkey;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;
use std::env;

/// all - 全コマンドライン引数を取得
/// 引数: なし
/// 例: (args/all) => ["./qi" "script.qi" "arg1" "arg2"]
pub fn native_args_all(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["args/all"]));
    }

    let cmd_args: Vec<Value> = env::args().map(Value::String).collect();
    Ok(Value::List(cmd_args.into()))
}

/// get - 指定位置の引数を取得
/// 引数: (index [default]) - 引数のインデックス（0 = プログラム名）、デフォルト値
/// 例: (args/get 0)           ;; プログラム名
///     (args/get 1)           ;; 第1引数
///     (args/get 5 "default") ;; 第5引数、なければ"default"
pub fn native_args_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["args/get"]));
    }

    let index = match &args[0] {
        Value::Integer(i) if *i >= 0 => *i as usize,
        Value::Integer(_) => {
            return Err(fmt_msg(MsgKey::MustBeNonNegative, &["args/get", "index"]))
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["args/get", "index"])),
    };

    let cmd_args: Vec<String> = env::args().collect();

    if index < cmd_args.len() {
        Ok(Value::String(cmd_args[index].clone()))
    } else if args.len() == 2 {
        Ok(args[1].clone())
    } else {
        Ok(Value::Nil)
    }
}

/// parse - コマンドライン引数をパース（フラグ・オプション・位置引数）
/// 引数: なし
/// 返値: {:flags [...] :options {...} :args [...]}
///
/// 解析ルール:
/// - "--flag" → フラグ（真偽値）
/// - "--key=value" または "--key value" → オプション（キー・値ペア）
/// - "-abc" → 短縮フラグ（a, b, c）
/// - その他 → 位置引数
///
/// 例: (args/parse)
///     プログラム実行: ./qi script.qi --verbose --port 3000 -df input.txt
///     結果: {:flags ["verbose" "d" "f"]
///           :options {"port" "3000"}
///           :args ["./qi" "script.qi" "input.txt"]}
pub fn native_args_parse(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["args/parse"]));
    }

    let cmd_args: Vec<String> = env::args().collect();

    let mut flags = Vec::new();
    let mut options = HashMap::new();
    let mut positional = Vec::new();

    let mut skip_next = false;
    for (i, arg) in cmd_args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if arg.starts_with("--") {
            // 長いオプション --key=value または --key value
            // starts_with("--") をチェック済みのため安全
            #[allow(clippy::expect_used)]
            let arg_trimmed = arg
                .strip_prefix("--")
                .expect("checked by starts_with above");

            if let Some(eq_pos) = arg_trimmed.find('=') {
                // --key=value形式
                let key = arg_trimmed[..eq_pos].to_string();
                let value = arg_trimmed[eq_pos + 1..].to_string();
                options.insert(key, Value::String(value));
            } else if i + 1 < cmd_args.len() && !cmd_args[i + 1].starts_with('-') {
                // --key value形式
                let key = arg_trimmed.to_string();
                let value = cmd_args[i + 1].clone();
                options.insert(key, Value::String(value));
                skip_next = true;
            } else {
                // フラグ
                flags.push(Value::String(arg_trimmed.to_string()));
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            // 短いオプション -abc → a, b, c
            for ch in arg[1..].chars() {
                flags.push(Value::String(ch.to_string()));
            }
        } else {
            // 位置引数
            positional.push(Value::String(arg.clone()));
        }
    }

    let mut result = HashMap::new();
    result.insert("flags".to_string(), Value::List(flags.into()));
    result.insert(
        "options".to_string(),
        Value::Map(convert_string_map_to_mapkey(options)),
    );
    result.insert("args".to_string(), Value::List(positional.into()));

    Ok(Value::Map(convert_string_map_to_mapkey(result)))
}

/// count - コマンドライン引数の数を取得
/// 引数: なし
/// 例: (args/count) => 5
pub fn native_args_count(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["args/count"]));
    }

    let count = env::args().count();
    Ok(Value::Integer(count as i64))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category args
/// @qi-doc:functions all, get, parse, count
pub const FUNCTIONS: super::NativeFunctions = &[
    ("args/all", native_args_all),
    ("args/get", native_args_get),
    ("args/parse", native_args_parse),
    ("args/count", native_args_count),
];
