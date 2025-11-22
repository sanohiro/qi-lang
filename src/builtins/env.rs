//! 環境変数関連関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::env;
use std::fs;

/// 保護された環境変数のリスト（上書きを警告）
const PROTECTED_VARS: &[&str] = &[
    "PATH",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "HOME",
    "USER",
    "SHELL",
    "TMPDIR",
];

/// 環境変数名が有効かチェック（英数字とアンダースコアのみ、先頭は英字）
fn is_valid_env_key(key: &str) -> bool {
    key.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 保護された変数の上書きを警告
fn warn_if_protected_var(key: &str) {
    if PROTECTED_VARS.contains(&key) {
        eprintln!("Warning: Overwriting protected system variable: {}", key);
    }
}

/// get - 環境変数を取得
/// 引数: (key [default]) - 環境変数名、オプションでデフォルト値
/// 例: (env/get "PATH")
///     (env/get "MISSING_VAR" "default-value")
pub fn native_env_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["env/get"]));
    }

    let key = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["env/get", "a string"])),
    };

    match env::var(key) {
        Ok(value) => Ok(Value::String(value)),
        Err(_) => {
            if args.len() == 2 {
                Ok(args[1].clone())
            } else {
                Ok(Value::Nil)
            }
        }
    }
}

/// set - 環境変数を設定
/// 引数: (key value) - 環境変数名と値
/// 例: (env/set "MY_VAR" "my-value")
pub fn native_env_set(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["env/set", "2"]));
    }

    let key = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["env/set", "a string"])),
    };

    let value = match &args[1] {
        Value::String(s) => s,
        Value::Integer(i) => &i.to_string(),
        Value::Float(f) => &f.to_string(),
        Value::Bool(b) => {
            if *b {
                "true"
            } else {
                "false"
            }
        }
        _ => return Err(fmt_msg(MsgKey::ValueMustBeStringNumberBool, &["env/set"])),
    };

    env::set_var(key, value);
    Ok(Value::Nil)
}

/// all - 全環境変数を取得
/// 引数: なし
/// 例: (env/all)
pub fn native_env_all(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["env/all"]));
    }

    let mut map = crate::new_hashmap();
    for (key, value) in env::vars() {
        map.insert(crate::value::MapKey::String(key), Value::String(value));
    }

    Ok(Value::Map(map))
}

/// load-dotenv - .envファイルを読み込んで環境変数に設定
/// 引数: ([path]) - .envファイルのパス（デフォルト: ".env"）
/// 例: (env/load-dotenv)
///     (env/load-dotenv ".env.local")
pub fn native_env_load_dotenv(args: &[Value]) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(fmt_msg(MsgKey::Need0Or1Args, &["env/load-dotenv"]));
    }

    let path = if args.is_empty() {
        ".env"
    } else {
        match &args[0] {
            Value::String(s) => s.as_str(),
            _ => {
                return Err(fmt_msg(
                    MsgKey::FirstArgMustBe,
                    &["env/load-dotenv", "a string"],
                ))
            }
        }
    };

    let content = fs::read_to_string(path)
        .map_err(|e| fmt_msg(MsgKey::EnvLoadDotenvFailedToRead, &[path, &e.to_string()]))?;

    let mut loaded_count = 0;
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // 空行とコメントをスキップ
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // KEY=VALUE形式をパース
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let mut value = line[eq_pos + 1..].trim();

            // クオートを除去
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = &value[1..value.len() - 1];
            }

            if !key.is_empty() {
                // キー名の検証（英数字とアンダースコアのみ、先頭は英字）
                if !is_valid_env_key(key) {
                    return Err(fmt_msg(
                        MsgKey::InvalidEnvVarName,
                        &["env/load-dotenv", key],
                    ));
                }

                // 保護された変数の上書きを警告
                warn_if_protected_var(key);

                env::set_var(key, value);
                loaded_count += 1;
            }
        } else {
            return Err(fmt_msg(
                MsgKey::EnvLoadDotenvInvalidFormat,
                &[&(line_num + 1).to_string(), line],
            ));
        }
    }

    Ok(Value::Integer(loaded_count))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category env
/// @qi-doc:functions get, set, all, load-dotenv
pub const FUNCTIONS: super::NativeFunctions = &[
    ("env/get", native_env_get),
    ("env/set", native_env_set),
    ("env/all", native_env_all),
    ("env/load-dotenv", native_env_load_dotenv),
];
