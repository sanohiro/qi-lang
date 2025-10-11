//! ログ関数

use crate::value::Value;
use chrono::Local;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::LazyLock;

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// ログフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

/// ログ設定
struct LogConfig {
    level: LogLevel,
    format: LogFormat,
}

static LOG_CONFIG: LazyLock<RwLock<LogConfig>> = LazyLock::new(|| {
    RwLock::new(LogConfig {
        level: LogLevel::Info,
        format: LogFormat::Text,
    })
});

/// ログ出力の内部実装
fn log_internal(level: LogLevel, message: &str, context: Option<HashMap<String, Value>>) {
    let config = LOG_CONFIG.read();

    // レベルフィルタ
    if level < config.level {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z");

    match config.format {
        LogFormat::Text => {
            // プレーンテキスト形式
            eprint!("[{}] {} {}", timestamp, level.as_str(), message);

            if let Some(ctx) = context {
                if !ctx.is_empty() {
                    eprint!(" |");
                    for (k, v) in ctx.iter() {
                        eprint!(" {}={}", k, value_to_string(v));
                    }
                }
            }
            eprintln!();
        }
        LogFormat::Json => {
            // JSON形式
            let mut json_obj = HashMap::new();
            json_obj.insert("timestamp".to_string(), format!("{}", timestamp));
            json_obj.insert("level".to_string(), level.as_str().to_string());
            json_obj.insert("message".to_string(), message.to_string());

            if let Some(ctx) = context {
                for (k, v) in ctx {
                    json_obj.insert(k, value_to_json_string(&v));
                }
            }

            // 簡易JSON出力
            eprint!("{{");
            let mut first = true;
            for (k, v) in json_obj.iter() {
                if !first {
                    eprint!(",");
                }
                eprint!("\"{}\":\"{}\"", k, v.replace('\"', "\\\""));
                first = false;
            }
            eprintln!("}}");
        }
    }
}

/// Value を文字列に変換（ログ表示用）
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Nil => "nil".to_string(),
        _ => format!("{:?}", v),
    }
}

/// Value を JSON文字列に変換
fn value_to_json_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Nil => "null".to_string(),
        _ => format!("{:?}", v),
    }
}

/// debug - DEBUGレベルのログ出力
/// 引数: (message [context]) - メッセージ、オプションでコンテキストマップ
/// 例: (log/debug "処理開始")
///     (log/debug "ユーザー情報" {:user-id 123 :name "Alice"})
pub fn native_log_debug(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("log/debug: 1 or 2 arguments required (message [context])".to_string());
    }

    let message = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/debug: message must be a string".to_string()),
    };

    let context = if args.len() == 2 {
        match &args[1] {
            Value::Map(m) => Some(m.clone()),
            _ => return Err("log/debug: context must be a map".to_string()),
        }
    } else {
        None
    };

    log_internal(LogLevel::Debug, message, context);
    Ok(Value::Nil)
}

/// info - INFOレベルのログ出力
/// 引数: (message [context]) - メッセージ、オプションでコンテキストマップ
/// 例: (log/info "サーバー起動" {:port 3000})
pub fn native_log_info(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("log/info: 1 or 2 arguments required (message [context])".to_string());
    }

    let message = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/info: message must be a string".to_string()),
    };

    let context = if args.len() == 2 {
        match &args[1] {
            Value::Map(m) => Some(m.clone()),
            _ => return Err("log/info: context must be a map".to_string()),
        }
    } else {
        None
    };

    log_internal(LogLevel::Info, message, context);
    Ok(Value::Nil)
}

/// warn - WARNレベルのログ出力
/// 引数: (message [context]) - メッセージ、オプションでコンテキストマップ
/// 例: (log/warn "接続タイムアウト" {:timeout-ms 5000})
pub fn native_log_warn(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("log/warn: 1 or 2 arguments required (message [context])".to_string());
    }

    let message = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/warn: message must be a string".to_string()),
    };

    let context = if args.len() == 2 {
        match &args[1] {
            Value::Map(m) => Some(m.clone()),
            _ => return Err("log/warn: context must be a map".to_string()),
        }
    } else {
        None
    };

    log_internal(LogLevel::Warn, message, context);
    Ok(Value::Nil)
}

/// error - ERRORレベルのログ出力
/// 引数: (message [context]) - メッセージ、オプションでコンテキストマップ
/// 例: (log/error "データベース接続失敗" {:error "connection refused"})
pub fn native_log_error(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("log/error: 1 or 2 arguments required (message [context])".to_string());
    }

    let message = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/error: message must be a string".to_string()),
    };

    let context = if args.len() == 2 {
        match &args[1] {
            Value::Map(m) => Some(m.clone()),
            _ => return Err("log/error: context must be a map".to_string()),
        }
    } else {
        None
    };

    log_internal(LogLevel::Error, message, context);
    Ok(Value::Nil)
}

/// set-level - ログレベルを設定
/// 引数: (level) - ログレベル ("debug", "info", "warn", "error")
/// 例: (log/set-level "debug")
pub fn native_log_set_level(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("log/set-level: exactly 1 argument required (level)".to_string());
    }

    let level_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/set-level: level must be a string".to_string()),
    };

    let level = LogLevel::from_str(level_str)
        .ok_or_else(|| format!("log/set-level: invalid level '{}' (valid: debug, info, warn, error)", level_str))?;

    LOG_CONFIG.write().level = level;
    Ok(Value::Nil)
}

/// set-format - ログフォーマットを設定
/// 引数: (format) - フォーマット ("text", "json")
/// 例: (log/set-format "json")
pub fn native_log_set_format(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("log/set-format: exactly 1 argument required (format)".to_string());
    }

    let format_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err("log/set-format: format must be a string".to_string()),
    };

    let format = match format_str.to_lowercase().as_str() {
        "text" | "plain" => LogFormat::Text,
        "json" => LogFormat::Json,
        _ => return Err(format!("log/set-format: invalid format '{}' (valid: text, json)", format_str)),
    };

    LOG_CONFIG.write().format = format;
    Ok(Value::Nil)
}
