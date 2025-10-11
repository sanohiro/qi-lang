//! 環境変数関連関数

use crate::value::Value;
use std::env;
use std::fs;
use std::collections::HashMap;

/// get - 環境変数を取得
/// 引数: (key [default]) - 環境変数名、オプションでデフォルト値
/// 例: (env/get "PATH")
///     (env/get "MISSING_VAR" "default-value")
pub fn native_env_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("env/get: 1 or 2 arguments required (key [default])".to_string());
    }

    let key = match &args[0] {
        Value::String(s) => s,
        _ => return Err("env/get: first argument must be a string".to_string()),
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
        return Err("env/set: exactly 2 arguments required (key value)".to_string());
    }

    let key = match &args[0] {
        Value::String(s) => s,
        _ => return Err("env/set: key must be a string".to_string()),
    };

    let value = match &args[1] {
        Value::String(s) => s,
        Value::Integer(i) => &i.to_string(),
        Value::Float(f) => &f.to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" },
        _ => return Err("env/set: value must be a string, number, or boolean".to_string()),
    };

    env::set_var(key, value);
    Ok(Value::Nil)
}

/// all - 全環境変数を取得
/// 引数: なし
/// 例: (env/all)
pub fn native_env_all(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("env/all: no arguments required".to_string());
    }

    let mut map = HashMap::new();
    for (key, value) in env::vars() {
        map.insert(key, Value::String(value));
    }

    Ok(Value::Map(map))
}

/// load-dotenv - .envファイルを読み込んで環境変数に設定
/// 引数: ([path]) - .envファイルのパス（デフォルト: ".env"）
/// 例: (env/load-dotenv)
///     (env/load-dotenv ".env.local")
pub fn native_env_load_dotenv(args: &[Value]) -> Result<Value, String> {
    if args.len() > 1 {
        return Err("env/load-dotenv: 0 or 1 argument required ([path])".to_string());
    }

    let path = if args.is_empty() {
        ".env"
    } else {
        match &args[0] {
            Value::String(s) => s.as_str(),
            _ => return Err("env/load-dotenv: path must be a string".to_string()),
        }
    };

    let content = fs::read_to_string(path)
        .map_err(|e| format!("env/load-dotenv: failed to read file '{}': {}", path, e))?;

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
            if (value.starts_with('"') && value.ends_with('"')) ||
               (value.starts_with('\'') && value.ends_with('\'')) {
                value = &value[1..value.len() - 1];
            }

            if !key.is_empty() {
                env::set_var(key, value);
                loaded_count += 1;
            }
        } else {
            return Err(format!(
                "env/load-dotenv: invalid format at line {}: '{}'",
                line_num + 1,
                line
            ));
        }
    }

    Ok(Value::Integer(loaded_count))
}
