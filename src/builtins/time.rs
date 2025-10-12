//! 日時処理関数
//!
//! このモジュールは `std-time` feature でコンパイルされます。

#![cfg(feature = "std-time")]

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};

/// now-iso - 現在時刻をISO 8601形式で取得
pub fn native_now_iso(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["time/now-iso"]));
    }

    let now = Utc::now();
    Ok(Value::String(now.to_rfc3339()))
}

/// from-unix - Unixタイムスタンプ（秒）をISO 8601形式に変換
pub fn native_from_unix(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/from-unix"]));
    }

    match &args[0] {
        Value::Integer(timestamp) => {
            match Utc.timestamp_opt(*timestamp, 0) {
                chrono::LocalResult::Single(dt) => Ok(Value::String(dt.to_rfc3339())),
                _ => Err(fmt_msg(MsgKey::InvalidTimestamp, &["time/from-unix"])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/from-unix", "integers"])),
    }
}

/// to-unix - ISO 8601形式の文字列をUnixタイムスタンプ（秒）に変換
pub fn native_to_unix(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/to-unix"]));
    }

    match &args[0] {
        Value::String(s) => {
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => Ok(Value::Integer(dt.timestamp())),
                Err(_) => Err(fmt_msg(MsgKey::InvalidDateFormat, &["time/to-unix", s])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/to-unix", "strings"])),
    }
}

/// format - タイムスタンプを指定フォーマットで文字列化
/// 第1引数: Unixタイムスタンプ（整数）またはISO 8601文字列
/// 第2引数: フォーマット文字列（strftime形式）
pub fn native_format(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/format"]));
    }

    let dt = match &args[0] {
        Value::Integer(timestamp) => {
            match Utc.timestamp_opt(*timestamp, 0) {
                chrono::LocalResult::Single(dt) => dt,
                _ => return Err(fmt_msg(MsgKey::InvalidTimestamp, &["time/format"])),
            }
        }
        Value::String(s) => {
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => return Err(fmt_msg(MsgKey::InvalidDateFormat, &["time/format", s])),
            }
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["time/format (timestamp)", "integers or strings"])),
    };

    match &args[1] {
        Value::String(format_str) => {
            let formatted = dt.format(format_str).to_string();
            Ok(Value::String(formatted))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["time/format (format)", "strings"])),
    }
}

/// today - 今日の日付をYYYY-MM-DD形式で取得
pub fn native_today(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["time/today"]));
    }

    let now = Utc::now();
    Ok(Value::String(now.format("%Y-%m-%d").to_string()))
}

/// add-days - 日付に日数を加算
/// 第1引数: Unixタイムスタンプ（整数）またはISO 8601文字列
/// 第2引数: 加算する日数（整数）
pub fn native_add_days(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/add-days"]));
    }

    let dt = parse_datetime(&args[0], "time/add-days")?;
    let days = parse_integer(&args[1], "time/add-days (days)")?;

    let new_dt = dt + Duration::days(days);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// add-hours - 日付に時間を加算
pub fn native_add_hours(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/add-hours"]));
    }

    let dt = parse_datetime(&args[0], "time/add-hours")?;
    let hours = parse_integer(&args[1], "time/add-hours (hours)")?;

    let new_dt = dt + Duration::hours(hours);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// add-minutes - 日付に分を加算
pub fn native_add_minutes(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/add-minutes"]));
    }

    let dt = parse_datetime(&args[0], "time/add-minutes")?;
    let minutes = parse_integer(&args[1], "time/add-minutes (minutes)")?;

    let new_dt = dt + Duration::minutes(minutes);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// sub-days - 日付から日数を減算
pub fn native_sub_days(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/sub-days"]));
    }

    let dt = parse_datetime(&args[0], "time/sub-days")?;
    let days = parse_integer(&args[1], "time/sub-days (days)")?;

    let new_dt = dt - Duration::days(days);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// sub-hours - 日付から時間を減算
pub fn native_sub_hours(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/sub-hours"]));
    }

    let dt = parse_datetime(&args[0], "time/sub-hours")?;
    let hours = parse_integer(&args[1], "time/sub-hours (hours)")?;

    let new_dt = dt - Duration::hours(hours);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// sub-minutes - 日付から分を減算
pub fn native_sub_minutes(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/sub-minutes"]));
    }

    let dt = parse_datetime(&args[0], "time/sub-minutes")?;
    let minutes = parse_integer(&args[1], "time/sub-minutes (minutes)")?;

    let new_dt = dt - Duration::minutes(minutes);
    Ok(Value::Integer(new_dt.timestamp()))
}

/// diff-days - 2つの日付の差を日数で取得
pub fn native_diff_days(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/diff-days"]));
    }

    let dt1 = parse_datetime(&args[0], "time/diff-days (date1)")?;
    let dt2 = parse_datetime(&args[1], "time/diff-days (date2)")?;

    let diff = dt1.signed_duration_since(dt2);
    Ok(Value::Integer(diff.num_days()))
}

/// diff-hours - 2つの日付の差を時間で取得
pub fn native_diff_hours(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/diff-hours"]));
    }

    let dt1 = parse_datetime(&args[0], "time/diff-hours (date1)")?;
    let dt2 = parse_datetime(&args[1], "time/diff-hours (date2)")?;

    let diff = dt1.signed_duration_since(dt2);
    Ok(Value::Integer(diff.num_hours()))
}

/// diff-minutes - 2つの日付の差を分で取得
pub fn native_diff_minutes(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/diff-minutes"]));
    }

    let dt1 = parse_datetime(&args[0], "time/diff-minutes (date1)")?;
    let dt2 = parse_datetime(&args[1], "time/diff-minutes (date2)")?;

    let diff = dt1.signed_duration_since(dt2);
    Ok(Value::Integer(diff.num_minutes()))
}

/// before? - date1がdate2より前か判定
pub fn native_before(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/before?"]));
    }

    let dt1 = parse_datetime(&args[0], "time/before? (date1)")?;
    let dt2 = parse_datetime(&args[1], "time/before? (date2)")?;

    Ok(Value::Bool(dt1 < dt2))
}

/// after? - date1がdate2より後か判定
pub fn native_after(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/after?"]));
    }

    let dt1 = parse_datetime(&args[0], "time/after? (date1)")?;
    let dt2 = parse_datetime(&args[1], "time/after? (date2)")?;

    Ok(Value::Bool(dt1 > dt2))
}

/// between? - dateがstart〜end内か判定
pub fn native_between(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need3Args, &["time/between?"]));
    }

    let date = parse_datetime(&args[0], "time/between? (date)")?;
    let start = parse_datetime(&args[1], "time/between? (start)")?;
    let end = parse_datetime(&args[2], "time/between? (end)")?;

    Ok(Value::Bool(date >= start && date <= end))
}

/// parse - フォーマット文字列を使って日付文字列をパース
pub fn native_parse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["time/parse"]));
    }

    let date_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["time/parse (date)", "strings"])),
    };

    let format_str = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["time/parse (format)", "strings"])),
    };

    match DateTime::parse_from_str(&format!("{} +0000", date_str), &format!("{} %z", format_str)) {
        Ok(dt) => Ok(Value::Integer(dt.timestamp())),
        Err(_) => Err(fmt_msg(MsgKey::TimeParseFailedToParse, &[date_str, format_str])),
    }
}

/// year - 日付から年を取得
pub fn native_year(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/year"]));
    }

    let dt = parse_datetime(&args[0], "time/year")?;
    Ok(Value::Integer(dt.year() as i64))
}

/// month - 日付から月を取得（1-12）
pub fn native_month(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/month"]));
    }

    let dt = parse_datetime(&args[0], "time/month")?;
    Ok(Value::Integer(dt.month() as i64))
}

/// day - 日付から日を取得（1-31）
pub fn native_day(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/day"]));
    }

    let dt = parse_datetime(&args[0], "time/day")?;
    Ok(Value::Integer(dt.day() as i64))
}

/// hour - 日付から時を取得（0-23）
pub fn native_hour(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/hour"]));
    }

    let dt = parse_datetime(&args[0], "time/hour")?;
    Ok(Value::Integer(dt.hour() as i64))
}

/// minute - 日付から分を取得（0-59）
pub fn native_minute(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/minute"]));
    }

    let dt = parse_datetime(&args[0], "time/minute")?;
    Ok(Value::Integer(dt.minute() as i64))
}

/// second - 日付から秒を取得（0-59）
pub fn native_second(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/second"]));
    }

    let dt = parse_datetime(&args[0], "time/second")?;
    Ok(Value::Integer(dt.second() as i64))
}

/// weekday - 日付から曜日を取得（0=日曜, 1=月曜, ..., 6=土曜）
pub fn native_weekday(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time/weekday"]));
    }

    let dt = parse_datetime(&args[0], "time/weekday")?;
    // chronoのweekday()は月曜=0なので、日曜=0に変換
    let wd = dt.weekday().num_days_from_sunday();
    Ok(Value::Integer(wd as i64))
}

// ========================================
// ヘルパー関数
// ========================================

/// DateTimeにパース（UnixタイムスタンプまたはISO 8601文字列）
fn parse_datetime(value: &Value, context: &str) -> Result<DateTime<Utc>, String> {
    match value {
        Value::Integer(timestamp) => {
            match Utc.timestamp_opt(*timestamp, 0) {
                chrono::LocalResult::Single(dt) => Ok(dt),
                _ => Err(fmt_msg(MsgKey::InvalidTimestamp, &[context])),
            }
        }
        Value::String(s) => {
            match DateTime::parse_from_rfc3339(s) {
                Ok(dt) => Ok(dt.with_timezone(&Utc)),
                Err(_) => Err(fmt_msg(MsgKey::InvalidDateFormat, &[context, s])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[context, "integers or strings"])),
    }
}

/// 整数にパース
fn parse_integer(value: &Value, context: &str) -> Result<i64, String> {
    match value {
        Value::Integer(n) => Ok(*n),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &[context, "integers"])),
    }
}
