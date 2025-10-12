//! YAMLモジュール
//!
//! YAML処理関数を提供:
//! - parse: YAML文字列をパース
//! - stringify: 値をYAML文字列に変換
//! - pretty: 値をYAML文字列に変換（整形済み、json/prettyとの互換性）

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use serde_yaml;

/// YAML文字列をパースしてQi値に変換
///
/// # 引数
/// - args[0]: YAML文字列
///
/// # 戻り値
/// - 成功時: {:ok パース結果}
/// - 失敗時: {:error エラーメッセージ}
pub fn native_parse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["yaml/parse"]));
    }

    let yaml_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["yaml/parse", "a string"])),
    };

    match serde_yaml::from_str::<serde_yaml::Value>(yaml_str) {
        Ok(yaml) => Ok(Value::Map(
            [(
                "ok".to_string(),
                yaml_to_value(yaml),
            )]
            .into_iter()
            .collect(),
        )),
        Err(e) => Ok(Value::Map(
            [(
                "error".to_string(),
                Value::String(format!("YAMLパースエラー: {}", e)),
            )]
            .into_iter()
            .collect(),
        )),
    }
}

/// Qi値をYAML文字列に変換
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: {:ok YAML文字列}
/// - 失敗時: {:error エラーメッセージ}
pub fn native_stringify(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["yaml/stringify"]));
    }

    match serde_yaml::to_string(&value_to_yaml(&args[0])) {
        Ok(s) => Ok(Value::Map(
            [("ok".to_string(), Value::String(s))]
                .into_iter()
                .collect(),
        )),
        Err(e) => Ok(Value::Map(
            [(
                "error".to_string(),
                Value::String(format!("YAML変換エラー: {}", e)),
            )]
            .into_iter()
            .collect(),
        )),
    }
}

/// Qi値をYAML文字列に変換（整形済み形式、json/prettyとの互換性のため用意）
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: {:ok YAML文字列（整形済み）}
/// - 失敗時: {:error エラーメッセージ}
///
/// # 注意
/// YAMLは常に整形されるため、yaml/stringifyと同じ結果を返す
pub fn native_pretty(args: &[Value]) -> Result<Value, String> {
    // YAMLは常に整形されるため、stringifyと同じ
    native_stringify(args)
}

/// serde_yaml::ValueをQi Valueに変換
fn yaml_to_value(yaml: serde_yaml::Value) -> Value {
    match yaml {
        serde_yaml::Value::Null => Value::Nil,
        serde_yaml::Value::Bool(b) => Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil
            }
        }
        serde_yaml::Value::String(s) => Value::String(s),
        serde_yaml::Value::Sequence(arr) => {
            Value::Vector(arr.into_iter().map(yaml_to_value).collect())
        }
        serde_yaml::Value::Mapping(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                // YAMLのキーは文字列に変換
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    _ => continue, // その他のキーはスキップ
                };
                map.insert(key, yaml_to_value(v));
            }
            Value::Map(map)
        }
        serde_yaml::Value::Tagged(_) => Value::Nil, // Taggedはサポート外
    }
}

/// Qi Valueをserde_yaml::Valueに変換
fn value_to_yaml(value: &Value) -> serde_yaml::Value {
    match value {
        Value::Nil => serde_yaml::Value::Null,
        Value::Bool(b) => serde_yaml::Value::Bool(*b),
        Value::Integer(i) => serde_yaml::Value::Number((*i).into()),
        Value::Float(f) => {
            serde_yaml::Value::Number(serde_yaml::Number::from(*f))
        }
        Value::String(s) => serde_yaml::Value::String(s.clone()),
        Value::Keyword(s) => serde_yaml::Value::String(s.clone()),
        Value::Vector(v) => {
            serde_yaml::Value::Sequence(v.iter().map(value_to_yaml).collect())
        }
        Value::List(l) => {
            serde_yaml::Value::Sequence(l.iter().map(value_to_yaml).collect())
        }
        Value::Map(m) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, v) in m.iter() {
                mapping.insert(
                    serde_yaml::Value::String(k.clone()),
                    value_to_yaml(v),
                );
            }
            serde_yaml::Value::Mapping(mapping)
        }
        _ => serde_yaml::Value::Null,
    }
}
