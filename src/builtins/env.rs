//! 環境変数関連関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::env;
use std::fs;

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
