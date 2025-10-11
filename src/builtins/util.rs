//! ユーティリティ関数（日付・時刻、JSON、型変換）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

/// now - 現在時刻（UNIXタイムスタンプ秒）
pub fn native_now(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["now"]));
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("now: system time error: {}", e))?;

    Ok(Value::Integer(duration.as_secs() as i64))
}

/// timestamp - 現在時刻（UNIXタイムスタンプミリ秒）
pub fn native_timestamp(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["timestamp"]));
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("timestamp: system time error: {}", e))?;

    Ok(Value::Integer(duration.as_millis() as i64))
}

/// sleep - 指定ミリ秒スリープ
pub fn native_sleep(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["sleep", "1", "(milliseconds)"]));
    }

    let millis = match &args[0] {
        Value::Integer(n) => {
            if *n < 0 {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["sleep", "duration"]));
            }
            *n as u64
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["sleep", "argument"])),
    };

    thread::sleep(Duration::from_millis(millis));
    Ok(Value::Nil)
}

/// json-parse - JSON文字列をパース
pub fn native_json_parse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedNArgsDesc, &["json-parse", "1", "(JSON string)"]));
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["json-parse", "a string"])),
    };

    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("json-parse: {}", e))?;

    json_to_value(&json_value)
}

/// json-stringify - 値をJSON文字列化
pub fn native_json_stringify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["json-stringify"]));
    }

    let json_value = value_to_json(&args[0])?;
    let json_str = serde_json::to_string(&json_value)
        .map_err(|e| format!("json-stringify: {}", e))?;

    Ok(Value::String(json_str))
}

/// json-pretty - 値を整形されたJSON文字列化
pub fn native_json_pretty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["json-pretty"]));
    }

    let json_value = value_to_json(&args[0])?;
    let json_str = serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("json-pretty: {}", e))?;

    Ok(Value::String(json_str))
}

// JSON変換ヘルパー
fn json_to_value(json: &serde_json::Value) -> Result<Value, String> {
    match json {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(fmt_msg(MsgKey::UnsupportedNumberType, &[]))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>, _> = arr.iter().map(json_to_value).collect();
            Ok(Value::List(values?))
        }
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), json_to_value(val)?);
            }
            Ok(Value::Map(map))
        }
    }
}

fn value_to_json(value: &Value) -> Result<serde_json::Value, String> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::json!(i)),
        Value::Float(f) => Ok(serde_json::json!(f)),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Keyword(k) => Ok(serde_json::Value::String(format!(":{}", k))),
        Value::List(items) | Value::Vector(items) => {
            let arr: Result<Vec<_>, _> = items.iter().map(value_to_json).collect();
            Ok(serde_json::Value::Array(arr?))
        }
        Value::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in m {
                obj.insert(key.clone(), value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err(format!("Cannot convert {:?} to JSON", value)),
    }
}

/// to-int - 値を整数に変換
pub fn native_to_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-int"]));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(*i)),
        Value::Float(f) => Ok(Value::Integer(*f as i64)),
        Value::String(s) => {
            s.parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| format!("to-int: cannot parse '{}' as integer", s))
        }
        Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        _ => Err(format!("to-int: cannot convert {:?} to integer", args[0])),
    }
}

/// to-float - 値を浮動小数点数に変換
pub fn native_to_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-float"]));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| format!("to-float: cannot parse '{}' as float", s))
        }
        _ => Err(format!("to-float: cannot convert {:?} to float", args[0])),
    }
}

/// to-string - 値を文字列に変換
pub fn native_to_string(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-string"]));
    }

    let s = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Keyword(k) => k.clone(),
        Value::Nil => "nil".to_string(),
        _ => format!("{:?}", args[0]),
    };

    Ok(Value::String(s))
}

/// number? - 数値判定
pub fn native_number_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["number?"]));
    }

    let is_number = matches!(args[0], Value::Integer(_) | Value::Float(_));
    Ok(Value::Bool(is_number))
}

/// function? - 関数判定
pub fn native_function_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["function?"]));
    }

    let is_function = matches!(args[0], Value::Function(_) | Value::NativeFunc(_));
    Ok(Value::Bool(is_function))
}

/// atom? - atom判定
pub fn native_atom_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["atom?"]));
    }

    let is_atom = matches!(args[0], Value::Atom(_));
    Ok(Value::Bool(is_atom))
}

/// _railway-pipe - Railway Oriented Programming用の内部関数
///
/// {:ok value}なら関数に渡し、{:error e}ならそのまま返す
pub fn native_railway_pipe(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["_railway-pipe"]));
    }

    let func = &args[0];
    let result = &args[1];

    // resultが{:ok value}または{:error ...}の形式かチェック
    match result {
        Value::Map(m) => {
            // {:ok value}の場合は値を取り出して関数に渡す
            if let Some(ok_val) = m.get("ok") {
                evaluator.apply_function(func, &[ok_val.clone()])
            }
            // {:error e}の場合はそのまま返す(ショートサーキット)
            else if m.contains_key("error") {
                Ok(result.clone())
            }
            else {
                Err(fmt_msg(MsgKey::RailwayRequiresOkError, &[]))
            }
        }
        _ => Err(fmt_msg(MsgKey::RailwayRequiresOkError, &[])),
    }
}

/// inspect - 値を整形して表示（デバッグ用）
pub fn native_inspect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["inspect"]));
    }

    println!("{}", pretty_print(&args[0], 0));
    Ok(args[0].clone())
}

/// 値を整形して表示するヘルパー関数
fn pretty_print(value: &Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match value {
        Value::Map(m) => {
            if m.is_empty() {
                return "{}".to_string();
            }
            let mut lines = vec!["{".to_string()];
            for (k, v) in m {
                lines.push(format!("{}  \"{}\": {}", indent_str, k, pretty_print(v, indent + 1)));
            }
            lines.push(format!("{}}}", indent_str));
            lines.join("\n")
        }
        Value::Vector(v) => {
            if v.is_empty() {
                return "[]".to_string();
            }
            let mut lines = vec!["[".to_string()];
            for item in v {
                lines.push(format!("{}  {}", indent_str, pretty_print(item, indent + 1)));
            }
            lines.push(format!("{}]", indent_str));
            lines.join("\n")
        }
        Value::List(l) => {
            if l.is_empty() {
                return "()".to_string();
            }
            let mut lines = vec!["(".to_string()];
            for item in l {
                lines.push(format!("{}  {}", indent_str, pretty_print(item, indent + 1)));
            }
            lines.push(format!("{})", indent_str));
            lines.join("\n")
        }
        Value::String(s) => format!("\"{}\"", s),
        _ => value.to_string()
    }
}

/// time - 関数実行時間を計測
pub fn native_time(args: &[Value], evaluator: &crate::eval::Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["time"]));
    }

    let start = std::time::Instant::now();
    let result = evaluator.apply_function(&args[0], &[])?;
    let elapsed = start.elapsed();

    println!("Elapsed: {:.3}s", elapsed.as_secs_f64());
    Ok(result)
}
