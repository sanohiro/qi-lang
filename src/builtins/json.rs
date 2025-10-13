//! JSONモジュール
//!
//! JSON処理関数を提供:
//! - parse: JSON文字列をパース
//! - stringify: 値をJSON文字列に変換（コンパクト）
//! - pretty: 値をJSON文字列に変換（整形済み）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use serde_json;

/// JSON文字列をパースしてQi値に変換
///
/// # 引数
/// - args[0]: JSON文字列
///
/// # 戻り値
/// - 成功時: {:ok パース結果}
/// - 失敗時: {:error エラーメッセージ}
pub fn native_parse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["json/parse"]));
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["json/parse", "a string"])),
    };

    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(json) => Ok(Value::Map(
            [("ok".to_string(), json_to_value(json))]
                .into_iter()
                .collect(),
        )),
        Err(e) => Ok(Value::Map(
            [(
                "error".to_string(),
                Value::String(format!("JSONパースエラー: {}", e)),
            )]
            .into_iter()
            .collect(),
        )),
    }
}

/// Qi値をJSON文字列に変換（コンパクト形式）
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: {:ok JSON文字列}
/// - 失敗時: {:error エラーメッセージ}
pub fn native_stringify(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["json/stringify"]));
    }

    match serde_json::to_string(&value_to_json(&args[0])) {
        Ok(s) => Ok(Value::Map(
            [("ok".to_string(), Value::String(s))].into_iter().collect(),
        )),
        Err(e) => Ok(Value::Map(
            [(
                "error".to_string(),
                Value::String(format!("JSON変換エラー: {}", e)),
            )]
            .into_iter()
            .collect(),
        )),
    }
}

/// Qi値をJSON文字列に変換（整形済み形式）
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: {:ok JSON文字列（整形済み）}
/// - 失敗時: {:error エラーメッセージ}
pub fn native_pretty(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["json/pretty"]));
    }

    match serde_json::to_string_pretty(&value_to_json(&args[0])) {
        Ok(s) => Ok(Value::Map(
            [("ok".to_string(), Value::String(s))].into_iter().collect(),
        )),
        Err(e) => Ok(Value::Map(
            [(
                "error".to_string(),
                Value::String(format!("JSON変換エラー: {}", e)),
            )]
            .into_iter()
            .collect(),
        )),
    }
}

/// serde_json::ValueをQi Valueに変換
fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Vector(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => Value::Map(
            obj.into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect(),
        ),
    }
}

/// Qi Valueをserde_json::Valueに変換
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Nil => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Keyword(s) => serde_json::Value::String(s.clone()),
        Value::Vector(v) => serde_json::Value::Array(v.iter().map(value_to_json).collect()),
        Value::List(l) => serde_json::Value::Array(l.iter().map(value_to_json).collect()),
        Value::Map(m) => serde_json::Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect(),
        ),
        _ => serde_json::Value::Null,
    }
}
