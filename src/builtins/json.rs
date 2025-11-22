//! JSONモジュール
//!
//! JSON処理関数を提供:
//! - parse: JSON文字列をパース
//! - stringify: 値をJSON文字列に変換（コンパクト）
//! - pretty: 値をJSON文字列に変換（整形済み）

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use serde_json;

/// JSON文字列をパースしてQi値に変換
///
/// # 引数
/// - args[0]: JSON文字列
///
/// # 戻り値
/// - 成功時: パース結果（値そのまま）
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
        Ok(json) => Ok(json_to_value(json)),
        Err(e) => Ok(Value::error(format!("JSONパースエラー: {}", e))),
    }
}

/// Qi値をJSON文字列に変換（コンパクト形式）
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: JSON文字列（値そのまま）
/// - 失敗時: {:error エラーメッセージ}
pub fn native_stringify(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "json/stringify");

    match serde_json::to_string(&value_to_json(&args[0])) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(format!("JSON変換エラー: {}", e))),
    }
}

/// Qi値をJSON文字列に変換（整形済み形式）
///
/// # 引数
/// - args[0]: 変換する値
///
/// # 戻り値
/// - 成功時: JSON文字列（整形済み、値そのまま）
/// - 失敗時: {:error エラーメッセージ}
pub fn native_pretty(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "json/pretty");

    match serde_json::to_string_pretty(&value_to_json(&args[0])) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Ok(Value::error(format!("JSON変換エラー: {}", e))),
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
                .map(|(k, v)| {
                    (
                        crate::value::MapKey::String(k),
                        json_to_value(v),
                    )
                })
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
        Value::Bytes(b) => {
            // バイナリデータはBase64エンコードして文字列として出力
            #[cfg(feature = "string-encoding")]
            {
                use base64::{engine::general_purpose, Engine as _};
                serde_json::Value::String(general_purpose::STANDARD.encode(b.as_ref()))
            }
            #[cfg(not(feature = "string-encoding"))]
            {
                // Base64が利用できない場合は、バイト配列として出力
                let arr: Vec<serde_json::Value> = b
                    .iter()
                    .map(|&byte| serde_json::Value::Number((byte as i64).into()))
                    .collect();
                serde_json::Value::Array(arr)
            }
        }
        Value::Keyword(s) => serde_json::Value::String(s.to_string()),
        Value::Vector(v) => {
            // サイズが分かっているので事前確保
            let mut arr = Vec::with_capacity(v.len());
            for item in v {
                arr.push(value_to_json(item));
            }
            serde_json::Value::Array(arr)
        }
        Value::List(l) => {
            // サイズが分かっているので事前確保
            let mut arr = Vec::with_capacity(l.len());
            for item in l {
                arr.push(value_to_json(item));
            }
            serde_json::Value::Array(arr)
        }
        Value::Map(m) => {
            // サイズが分かっているので事前確保
            let mut obj = serde_json::Map::with_capacity(m.len());
            for (k, v) in m {
                // MapKeyからJSONキーに変換
                let json_key = match k {
                    crate::value::MapKey::Keyword(kw) => {
                        // キーワードキー :name → "name"
                        kw.to_string()
                    }
                    crate::value::MapKey::String(s) => {
                        // 文字列キー "\"test\"" → "test"
                        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                            s[1..s.len() - 1].to_string()
                        } else {
                            s.clone()
                        }
                    }
                    crate::value::MapKey::Symbol(sym) => {
                        // シンボルキー → そのまま
                        sym.to_string()
                    }
                    crate::value::MapKey::Integer(i) => {
                        // 整数キー → 文字列化
                        i.to_string()
                    }
                };
                obj.insert(json_key, value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category data/json
/// @qi-doc:functions parse, stringify, pretty
pub const FUNCTIONS: super::NativeFunctions = &[
    ("json/parse", native_parse),
    ("json/stringify", native_stringify),
    ("json/pretty", native_pretty),
];
