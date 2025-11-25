//! ログ関数

use crate::builtins::value_helpers::{get_map_arg, get_string_ref};
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{MapKey, Value};
use parking_lot::RwLock;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Unix秒をISO8601風の文字列にフォーマット（簡易版）
fn format_unix_timestamp(secs: u64, millis: u32) -> String {
    // 簡易的な日時計算（うるう秒は考慮しない）
    const SECS_PER_DAY: u64 = 86400;
    const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let days_since_epoch = secs / SECS_PER_DAY;
    let time_of_day = secs % SECS_PER_DAY;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // 1970年からの年数計算（簡易版）
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    // 月と日の計算
    let mut month = 1;
    for &days_in_month in &DAYS_IN_MONTH {
        let adjusted_days = if month == 2 && is_leap_year(year) {
            days_in_month + 1
        } else {
            days_in_month
        };

        if remaining_days < adjusted_days {
            break;
        }
        remaining_days -= adjusted_days;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        year, month, day, hours, minutes, seconds, millis
    )
}

/// うるう年判定
fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// ログ出力の内部実装
fn log_internal(level: LogLevel, message: &str, context: Option<crate::HashMap<MapKey, Value>>) {
    let config = LOG_CONFIG.read();

    // レベルフィルタ
    if level < config.level {
        return;
    }

    // 標準ライブラリでタイムスタンプを生成（ISO8601風）
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let millis = duration.subsec_millis();
            // 簡易的なISO8601フォーマット（UTC）
            format!("{}+0000", format_unix_timestamp(secs, millis))
        }
        Err(_) => "1970-01-01T00:00:00.000+0000".to_string(),
    };

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
            let mut json_obj = std::collections::HashMap::new();
            json_obj.insert("timestamp".to_string(), timestamp.to_string());
            json_obj.insert("level".to_string(), level.as_str().to_string());
            json_obj.insert("message".to_string(), message.to_string());

            if let Some(ctx) = context {
                for (k, v) in ctx {
                    json_obj.insert(k.to_string(), value_to_json_string(&v));
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
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["log/debug"]));
    }

    let message = get_string_ref(args, 0, "log/debug")?;

    let context = if args.len() == 2 {
        Some(get_map_arg(args, 1, "log/debug")?)
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
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["log/info"]));
    }

    let message = get_string_ref(args, 0, "log/info")?;

    let context = if args.len() == 2 {
        Some(get_map_arg(args, 1, "log/info")?)
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
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["log/warn"]));
    }

    let message = get_string_ref(args, 0, "log/warn")?;

    let context = if args.len() == 2 {
        Some(get_map_arg(args, 1, "log/warn")?)
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
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["log/error"]));
    }

    let message = get_string_ref(args, 0, "log/error")?;

    let context = if args.len() == 2 {
        Some(get_map_arg(args, 1, "log/error")?)
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
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["log/set-level", "1"]));
    }

    let level_str = get_string_ref(args, 0, "log/set-level")?;

    let level = LogLevel::from_str(level_str)
        .ok_or_else(|| fmt_msg(MsgKey::LogSetLevelInvalidLevel, &[level_str]))?;

    LOG_CONFIG.write().level = level;
    Ok(Value::Nil)
}

/// set-format - ログフォーマットを設定
/// 引数: (format) - フォーマット ("text", "json")
/// 例: (log/set-format "json")
pub fn native_log_set_format(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["log/set-format", "1"]));
    }

    let format_str = get_string_ref(args, 0, "log/set-format")?;

    let format = match format_str.to_lowercase().as_str() {
        "text" | "plain" => LogFormat::Text,
        "json" => LogFormat::Json,
        _ => return Err(fmt_msg(MsgKey::LogSetFormatInvalidFormat, &[format_str])),
    };

    LOG_CONFIG.write().format = format;
    Ok(Value::Nil)
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category log
/// @qi-doc:functions debug, info, warn, error, set-level, set-format
pub const FUNCTIONS: super::NativeFunctions = &[
    ("log/debug", native_log_debug),
    ("log/info", native_log_info),
    ("log/warn", native_log_warn),
    ("log/error", native_log_error),
    ("log/set-level", native_log_set_level),
    ("log/set-format", native_log_set_format),
];
