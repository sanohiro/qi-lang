//! バリデーション関数
//!
//! スキーマベースのデータ検証機能を提供します。
//!
//! ## 主な機能
//! - 型チェック（:type）
//! - 必須フィールドチェック（:required）
//! - 数値範囲チェック（:min, :max）
//! - 文字列長チェック（:min-length, :max-length）
//! - パターンマッチ（:pattern）
//!
//! ## 使用例
//! ```qi
//! (def schema {:type "string" :min-length 3 :max-length 10})
//! (validate schema "hello")  ;=> {:ok "hello"}
//! (validate schema "ab")     ;=> {:error {:code "min-length" :message "..."}}
//! ```

use crate::constants::keywords::ERROR_KEY;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use crate::HashMap;
use regex::Regex;

/// validate関数
///
/// スキーマに基づいてデータを検証する
///
/// (validate schema data) => {:ok data} | {:error {...}}
pub fn native_validate(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["validate"]));
    }

    let schema = &args[0];
    let data = &args[1];

    validate_value(schema, data, None)
}

/// 値をスキーマで検証する（内部関数）
fn validate_value(schema: &Value, data: &Value, field_name: Option<&str>) -> Result<Value, String> {
    let Value::Map(schema_map) = schema else {
        return Err(fmt_msg(MsgKey::MustBeMap, &["validate", "schema"]));
    };

    // :required チェック
    let is_required = matches!(schema_map.get(":required"), Some(Value::Bool(true)));

    if matches!(data, Value::Nil) {
        if is_required {
            return Ok(error_result(
                field_name,
                "required",
                fmt_msg(MsgKey::ValidateRequired, &[]),
            ));
        } else {
            // オプショナルフィールドでnilの場合は成功
            let mut result_map = crate::new_hashmap();
            result_map.insert(":ok".to_string(), data.clone());
            return Ok(Value::Map(result_map));
        }
    }

    // :type チェック（nilでない場合のみ）
    if let Some(type_val) = schema_map.get(":type") {
        if let Some(error) = validate_type(type_val, data, field_name)? {
            return Ok(error);
        }
    }

    // 型別の詳細バリデーション
    match data {
        Value::String(s) => {
            if let Some(error) = validate_string(schema_map, s, field_name)? {
                return Ok(error);
            }
        }
        Value::Integer(n) => {
            if let Some(error) = validate_number(schema_map, *n as f64, field_name)? {
                return Ok(error);
            }
        }
        Value::Float(f) => {
            if let Some(error) = validate_number(schema_map, *f, field_name)? {
                return Ok(error);
            }
        }
        Value::List(_) | Value::Vector(_) => {
            if let Some(error) = validate_collection(schema_map, data, field_name)? {
                return Ok(error);
            }
        }
        Value::Map(m) => {
            if let Some(error) = validate_map(schema_map, m, field_name)? {
                return Ok(error);
            }
        }
        _ => {}
    }

    // 全て通過したら成功
    let mut result_map = crate::new_hashmap();
    result_map.insert(":ok".to_string(), data.clone());
    Ok(Value::Map(result_map))
}

/// 型チェック
fn validate_type(
    type_val: &Value,
    data: &Value,
    field_name: Option<&str>,
) -> Result<Option<Value>, String> {
    let Value::String(type_name) = type_val else {
        return Err(fmt_msg(
            MsgKey::ArgMustBeType,
            &["validate :type", "string"],
        ));
    };

    let expected = type_name.as_str();
    let matches = match expected {
        "string" => matches!(data, Value::String(_)),
        "number" => matches!(data, Value::Integer(_) | Value::Float(_)),
        "integer" => matches!(data, Value::Integer(_)),
        "boolean" => matches!(data, Value::Bool(_)),
        "map" => matches!(data, Value::Map(_)),
        "vector" => matches!(data, Value::Vector(_)),
        "list" => matches!(data, Value::List(_)),
        "keyword" => matches!(data, Value::Keyword(_)),
        "symbol" => matches!(data, Value::Symbol(_)),
        "nil" => matches!(data, Value::Nil),
        "any" => true,
        _ => {
            return Err(fmt_msg(
                MsgKey::ValidateTypeMismatch,
                &[&format!("unknown type: {}", expected)],
            ))
        }
    };

    if !matches {
        return Ok(Some(error_result(
            field_name,
            "type-mismatch",
            fmt_msg(MsgKey::ValidateTypeMismatch, &[expected]),
        )));
    }

    Ok(None)
}

/// 文字列バリデーション
fn validate_string(
    schema: &HashMap<MapKey, Value>,
    s: &str,
    field_name: Option<&str>,
) -> Result<Option<Value>, String> {
    // :min-length チェック
    if let Some(Value::Integer(min)) = schema.get(":min-length") {
        if s.chars().count() < *min as usize {
            return Ok(Some(error_result(
                field_name,
                "min-length",
                fmt_msg(MsgKey::ValidateMinLength, &[&min.to_string()]),
            )));
        }
    }

    // :max-length チェック
    if let Some(Value::Integer(max)) = schema.get(":max-length") {
        if s.chars().count() > *max as usize {
            return Ok(Some(error_result(
                field_name,
                "max-length",
                fmt_msg(MsgKey::ValidateMaxLength, &[&max.to_string()]),
            )));
        }
    }

    // :pattern チェック
    if let Some(Value::String(pattern)) = schema.get(":pattern") {
        let re = Regex::new(pattern)
            .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["validate", &e.to_string()]))?;
        if !re.is_match(s) {
            return Ok(Some(error_result(
                field_name,
                "pattern",
                fmt_msg(MsgKey::ValidatePattern, &[pattern]),
            )));
        }
    }

    Ok(None)
}

/// 数値バリデーション
fn validate_number(
    schema: &HashMap<MapKey, Value>,
    n: f64,
    field_name: Option<&str>,
) -> Result<Option<Value>, String> {
    // :min チェック
    if let Some(Value::Integer(min)) = schema.get(":min") {
        if n < *min as f64 {
            return Ok(Some(error_result(
                field_name,
                "min-value",
                fmt_msg(MsgKey::ValidateMinValue, &[&min.to_string()]),
            )));
        }
    }
    if let Some(Value::Float(min)) = schema.get(":min") {
        if n < *min {
            return Ok(Some(error_result(
                field_name,
                "min-value",
                fmt_msg(MsgKey::ValidateMinValue, &[&min.to_string()]),
            )));
        }
    }

    // :max チェック
    if let Some(Value::Integer(max)) = schema.get(":max") {
        if n > *max as f64 {
            return Ok(Some(error_result(
                field_name,
                "max-value",
                fmt_msg(MsgKey::ValidateMaxValue, &[&max.to_string()]),
            )));
        }
    }
    if let Some(Value::Float(max)) = schema.get(":max") {
        if n > *max {
            return Ok(Some(error_result(
                field_name,
                "max-value",
                fmt_msg(MsgKey::ValidateMaxValue, &[&max.to_string()]),
            )));
        }
    }

    // :positive チェック
    if let Some(Value::Bool(true)) = schema.get(":positive") {
        if n <= 0.0 {
            return Ok(Some(error_result(
                field_name,
                "positive",
                fmt_msg(MsgKey::MustBePositive, &["validate", "value"]),
            )));
        }
    }

    // :integer チェック
    if let Some(Value::Bool(true)) = schema.get(":integer") {
        if n.fract() != 0.0 {
            return Ok(Some(error_result(
                field_name,
                "integer",
                fmt_msg(MsgKey::MustBeInteger, &["validate", "value"]),
            )));
        }
    }

    Ok(None)
}

/// コレクションバリデーション
fn validate_collection(
    schema: &HashMap<MapKey, Value>,
    data: &Value,
    field_name: Option<&str>,
) -> Result<Option<Value>, String> {
    let items = match data {
        Value::List(l) => l.len(),
        Value::Vector(v) => v.len(),
        _ => return Ok(None),
    };

    // :min-items チェック
    if let Some(Value::Integer(min)) = schema.get(":min-items") {
        if items < *min as usize {
            return Ok(Some(error_result(
                field_name,
                "min-items",
                fmt_msg(MsgKey::ValidateMinItems, &[&min.to_string()]),
            )));
        }
    }

    // :max-items チェック
    if let Some(Value::Integer(max)) = schema.get(":max-items") {
        if items > *max as usize {
            return Ok(Some(error_result(
                field_name,
                "max-items",
                fmt_msg(MsgKey::ValidateMaxItems, &[&max.to_string()]),
            )));
        }
    }

    Ok(None)
}

/// マップバリデーション
fn validate_map(
    schema: &HashMap<MapKey, Value>,
    data: &HashMap<MapKey, Value>,
    _field_name: Option<&str>,
) -> Result<Option<Value>, String> {
    // :fields チェック
    if let Some(Value::Map(fields_schema)) = schema.get(":fields") {
        for (field_key, field_schema) in fields_schema.iter() {
            let field_data = data.get(field_key).unwrap_or(&Value::Nil);
            let result = validate_value(field_schema, field_data, Some(field_key))?;

            // エラーがあれば即座に返す
            if let Value::Map(m) = &result {
                if m.contains_key(&crate::constants::keywords::error_mapkey()) {
                    return Ok(Some(result));
                }
            }
        }
    }

    Ok(None)
}

/// エラー結果を生成
fn error_result(field: Option<&str>, code: &str, message: String) -> Value {
    let mut error_map = crate::new_hashmap();
    error_map.insert(":code".to_string(), Value::String(code.to_string()));
    error_map.insert(":message".to_string(), Value::String(message));

    if let Some(f) = field {
        error_map.insert(":field".to_string(), Value::String(f.to_string()));
    }

    let mut result_map = crate::new_hashmap();
    result_map.insert(ERROR_KEY.to_string(), Value::Map(error_map));
    Value::Map(result_map)
}

/// @qi-doc:category validation
/// @qi-doc:functions validate
/// @qi-doc:note スキーマベースのデータ検証
pub const FUNCTIONS: super::NativeFunctions = &[("validate", native_validate)];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use im::HashMap;

    /// ヘルパー関数: スキーママップを作成
    fn make_schema(entries: Vec<(&str, Value)>) -> Value {
        let mut map = crate::new_hashmap();
        for (key, val) in entries {
            map.insert(key.to_string(), val);
        }
        Value::Map(map)
    }

    /// ヘルパー関数: 成功結果かどうかをチェック
    fn is_ok(result: &Value) -> bool {
        if let Value::Map(m) = result {
            m.contains_key(":ok")
        } else {
            false
        }
    }

    /// ヘルパー関数: エラー結果かどうかをチェック
    fn is_error(result: &Value) -> bool {
        if let Value::Map(m) = result {
            m.contains_key(&crate::constants::keywords::error_mapkey())
        } else {
            false
        }
    }

    /// ヘルパー関数: エラーコードを取得
    fn get_error_code(result: &Value) -> Option<String> {
        if let Value::Map(m) = result {
            if let Some(Value::Map(error_map)) = m.get(&crate::constants::keywords::error_mapkey()) {
                if let Some(Value::String(code)) = error_map.get(":code") {
                    return Some(code.clone());
                }
            }
        }
        None
    }

    #[test]
    fn test_validate_type_string() {
        let schema = make_schema(vec![(":type", Value::String("string".to_string()))]);
        let data = Value::String("hello".to_string());
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for valid string");

        let invalid_data = Value::Integer(123);
        let result = native_validate(&[schema, invalid_data]).unwrap();
        assert!(is_error(&result), "Expected error for invalid type");
        assert_eq!(get_error_code(&result), Some("type-mismatch".to_string()));
    }

    #[test]
    fn test_validate_type_integer() {
        let schema = make_schema(vec![(":type", Value::String("integer".to_string()))]);
        let data = Value::Integer(42);
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for valid integer");

        let invalid_data = Value::Float(3.14);
        let result = native_validate(&[schema, invalid_data]).unwrap();
        assert!(
            is_error(&result),
            "Expected error for float instead of integer"
        );
    }

    #[test]
    fn test_validate_required() {
        let schema = make_schema(vec![
            (":type", Value::String("string".to_string())),
            (":required", Value::Bool(true)),
        ]);

        let data = Value::String("test".to_string());
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for non-nil value");

        let nil_data = Value::Nil;
        let result = native_validate(&[schema, nil_data]).unwrap();
        assert!(
            is_error(&result),
            "Expected error for required field with nil"
        );
        assert_eq!(get_error_code(&result), Some("required".to_string()));
    }

    #[test]
    fn test_validate_optional() {
        let schema = make_schema(vec![(":type", Value::String("string".to_string()))]);

        let nil_data = Value::Nil;
        let result = native_validate(&[schema, nil_data]).unwrap();
        assert!(
            is_ok(&result),
            "Expected success for optional field with nil"
        );
    }

    #[test]
    fn test_validate_string_length() {
        let schema = make_schema(vec![
            (":type", Value::String("string".to_string())),
            (":min-length", Value::Integer(3)),
            (":max-length", Value::Integer(10)),
        ]);

        // 正常な長さ
        let data = Value::String("hello".to_string());
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for valid length");

        // 短すぎる
        let short_data = Value::String("ab".to_string());
        let result = native_validate(&[schema.clone(), short_data]).unwrap();
        assert!(is_error(&result), "Expected error for too short string");
        assert_eq!(get_error_code(&result), Some("min-length".to_string()));

        // 長すぎる
        let long_data = Value::String("verylongstring".to_string());
        let result = native_validate(&[schema, long_data]).unwrap();
        assert!(is_error(&result), "Expected error for too long string");
        assert_eq!(get_error_code(&result), Some("max-length".to_string()));
    }

    #[test]
    fn test_validate_number_range() {
        let schema = make_schema(vec![
            (":type", Value::String("integer".to_string())),
            (":min", Value::Integer(0)),
            (":max", Value::Integer(100)),
        ]);

        // 範囲内
        let data = Value::Integer(50);
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for value in range");

        // 小さすぎる
        let small_data = Value::Integer(-5);
        let result = native_validate(&[schema.clone(), small_data]).unwrap();
        assert!(is_error(&result), "Expected error for value below min");
        assert_eq!(get_error_code(&result), Some("min-value".to_string()));

        // 大きすぎる
        let large_data = Value::Integer(150);
        let result = native_validate(&[schema, large_data]).unwrap();
        assert!(is_error(&result), "Expected error for value above max");
        assert_eq!(get_error_code(&result), Some("max-value".to_string()));
    }

    #[test]
    fn test_validate_pattern() {
        let schema = make_schema(vec![
            (":type", Value::String("string".to_string())),
            (":pattern", Value::String(r"^[a-z]+$".to_string())),
        ]);

        // パターンに一致
        let data = Value::String("hello".to_string());
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for matching pattern");

        // パターンに不一致
        let invalid_data = Value::String("Hello123".to_string());
        let result = native_validate(&[schema, invalid_data]).unwrap();
        assert!(is_error(&result), "Expected error for non-matching pattern");
        assert_eq!(get_error_code(&result), Some("pattern".to_string()));
    }

    #[test]
    fn test_validate_collection_items() {
        let schema = make_schema(vec![
            (":type", Value::String("vector".to_string())),
            (":min-items", Value::Integer(1)),
            (":max-items", Value::Integer(5)),
        ]);

        // 正常な個数
        let data = Value::Vector(im::Vector::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for valid item count");

        // 少なすぎる
        let empty_data = Value::Vector(im::Vector::new());
        let result = native_validate(&[schema.clone(), empty_data]).unwrap();
        assert!(is_error(&result), "Expected error for too few items");
        assert_eq!(get_error_code(&result), Some("min-items".to_string()));

        // 多すぎる
        let many_data = Value::Vector(im::Vector::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
            Value::Integer(6),
        ]));
        let result = native_validate(&[schema, many_data]).unwrap();
        assert!(is_error(&result), "Expected error for too many items");
        assert_eq!(get_error_code(&result), Some("max-items".to_string()));
    }

    #[test]
    fn test_validate_nested_map() {
        // ネストしたマップのスキーマ
        let mut name_schema = crate::new_hashmap();
        name_schema.insert(":type".to_string(), Value::String("string".to_string()));
        name_schema.insert(":required".to_string(), Value::Bool(true));
        name_schema.insert(":min-length".to_string(), Value::Integer(1));

        let mut age_schema = crate::new_hashmap();
        age_schema.insert(":type".to_string(), Value::String("integer".to_string()));
        age_schema.insert(":min".to_string(), Value::Integer(0));
        age_schema.insert(":max".to_string(), Value::Integer(150));

        let mut fields = crate::new_hashmap();
        fields.insert(":name".to_string(), Value::Map(name_schema));
        fields.insert(":age".to_string(), Value::Map(age_schema));

        let schema = make_schema(vec![
            (":type", Value::String("map".to_string())),
            (":fields", Value::Map(fields)),
        ]);

        // 正常なデータ
        let mut valid_data = crate::new_hashmap();
        valid_data.insert(":name".to_string(), Value::String("太郎".to_string()));
        valid_data.insert(":age".to_string(), Value::Integer(25));
        let result = native_validate(&[schema.clone(), Value::Map(valid_data)]).unwrap();
        assert!(is_ok(&result), "Expected success for valid nested map");

        // nameが空文字列（min-lengthエラー）
        let mut invalid_name = crate::new_hashmap();
        invalid_name.insert(":name".to_string(), Value::String("".to_string()));
        invalid_name.insert(":age".to_string(), Value::Integer(25));
        let result = native_validate(&[schema.clone(), Value::Map(invalid_name)]).unwrap();
        assert!(is_error(&result), "Expected error for empty name");
        assert_eq!(get_error_code(&result), Some("min-length".to_string()));

        // nameがない（requiredエラー）
        let mut missing_name = crate::new_hashmap();
        missing_name.insert(":age".to_string(), Value::Integer(25));
        let result = native_validate(&[schema.clone(), Value::Map(missing_name)]).unwrap();
        assert!(
            is_error(&result),
            "Expected error for missing required name"
        );
        assert_eq!(get_error_code(&result), Some("required".to_string()));

        // ageが範囲外（max-valueエラー）
        let mut invalid_age = crate::new_hashmap();
        invalid_age.insert(":name".to_string(), Value::String("太郎".to_string()));
        invalid_age.insert(":age".to_string(), Value::Integer(200));
        let result = native_validate(&[schema, Value::Map(invalid_age)]).unwrap();
        assert!(is_error(&result), "Expected error for age above max");
        assert_eq!(get_error_code(&result), Some("max-value".to_string()));
    }

    #[test]
    fn test_validate_positive_number() {
        let schema = make_schema(vec![
            (":type", Value::String("number".to_string())),
            (":positive", Value::Bool(true)),
        ]);

        // 正の数
        let data = Value::Float(3.14);
        let result = native_validate(&[schema.clone(), data]).unwrap();
        assert!(is_ok(&result), "Expected success for positive number");

        // ゼロ（エラー）
        let zero_data = Value::Integer(0);
        let result = native_validate(&[schema.clone(), zero_data]).unwrap();
        assert!(is_error(&result), "Expected error for zero (not positive)");
        assert_eq!(get_error_code(&result), Some("positive".to_string()));

        // 負の数
        let negative_data = Value::Float(-1.5);
        let result = native_validate(&[schema, negative_data]).unwrap();
        assert!(is_error(&result), "Expected error for negative number");
    }

    #[test]
    fn test_validate_any_type() {
        let schema = make_schema(vec![(":type", Value::String("any".to_string()))]);

        // 任意の型を受け入れる
        let values = vec![
            Value::String("test".to_string()),
            Value::Integer(42),
            Value::Float(3.14),
            Value::Bool(true),
            Value::Nil,
            Value::Vector(im::Vector::new()),
        ];

        for val in values {
            let result = native_validate(&[schema.clone(), val]).unwrap();
            assert!(is_ok(&result), "Expected success for any type");
        }
    }
}
