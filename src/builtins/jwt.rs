//! JWT（JSON Web Token）認証機能
//!
//! このモジュールは `auth-jwt` feature でコンパイルされます。

use crate::builtins::util::to_map_key;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{MapKey, Value};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value as JsonValue;

/// jwt/sign - JWTトークンを生成
///
/// 引数:
/// - payload: マップ（クレーム）
/// - secret: シークレットキー（文字列）
/// - algorithm: アルゴリズム（オプション、デフォルト: "HS256"）
/// - exp: 有効期限（秒数、オプション）
///
/// 戻り値: トークン文字列 または {:error message}
pub fn native_jwt_sign(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["jwt/sign", "2-4"]));
    }

    // ペイロード（マップ）
    let payload = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["jwt/sign (payload)", "maps"])),
    };

    // シークレットキー
    let secret = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["jwt/sign (secret)", "strings"])),
    };

    // アルゴリズム（オプション、デフォルト: HS256）
    let algorithm = if args.len() >= 3 {
        match &args[2] {
            Value::String(s) => parse_algorithm(s.as_str())?,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["jwt/sign (algorithm)", "strings"],
                ))
            }
        }
    } else {
        Algorithm::HS256
    };

    // 有効期限（オプション、秒数）
    let exp_seconds = if args.len() == 4 {
        match &args[3] {
            Value::Integer(i) => Some(*i),
            _ => return Err(fmt_msg(MsgKey::TypeOnly, &["jwt/sign (exp)", "integers"])),
        }
    } else {
        None
    };

    // QiのマップをJSON Valueに変換
    let mut claims = qi_map_to_json(payload)?;

    // 有効期限を追加
    if let Some(exp_secs) = exp_seconds {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time should be after UNIX_EPOCH")
            .as_secs();
        let exp = now + (exp_secs as u64);
        if let JsonValue::Object(ref mut map) = claims {
            map.insert("exp".to_string(), JsonValue::Number(exp.into()));
        }
    }

    // JWTトークン生成
    let header = Header::new(algorithm);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    match encode(&header, &claims, &encoding_key) {
        Ok(token) => Ok(Value::String(token)),
        Err(e) => Ok(Value::error(e.to_string())),
    }
}

/// jwt/verify - JWTトークンを検証してペイロードを取得
///
/// 引数:
/// - token: JWTトークン（文字列）
/// - secret: シークレットキー（文字列）
/// - algorithm: アルゴリズム（オプション、デフォルト: "HS256"）
///
/// 戻り値: ペイロードマップ または {:error message}
pub fn native_jwt_verify(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["jwt/verify", "2-3"]));
    }

    // トークン
    let token = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["jwt/verify (token)", "strings"],
            ))
        }
    };

    // シークレットキー
    let secret = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["jwt/verify (secret)", "strings"],
            ))
        }
    };

    // アルゴリズム（オプション、デフォルト: HS256）
    let algorithm = if args.len() == 3 {
        match &args[2] {
            Value::String(s) => parse_algorithm(s.as_str())?,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["jwt/verify (algorithm)", "strings"],
                ))
            }
        }
    } else {
        Algorithm::HS256
    };

    // JWTトークン検証
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(algorithm);
    validation.validate_exp = false; // 有効期限チェックはデフォルトで無効（expクレームがオプション）
    validation.required_spec_claims.clear(); // 必須クレームをクリア

    match decode::<JsonValue>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let payload = json_to_qi_value(&token_data.claims)?;
            Ok(payload)
        }
        Err(e) => Ok(Value::error(e.to_string())),
    }
}

/// jwt/decode - JWTトークンをデコード（検証なし）
///
/// 引数:
/// - token: JWTトークン（文字列）
///
/// 戻り値: {:header ... :payload ...} マップ または {:error message}
pub fn native_jwt_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["jwt/decode"]));
    }

    // トークン
    let token = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["jwt/decode (token)", "strings"],
            ))
        }
    };

    // デコード（検証なし）
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.required_spec_claims.clear(); // 必須クレームをクリア

    match decode::<JsonValue>(token, &DecodingKey::from_secret(&[]), &validation) {
        Ok(token_data) => {
            let header = json_to_qi_value(
                &serde_json::to_value(&token_data.header)
                    .expect("JWT header should always be serializable"),
            )?;
            let payload = json_to_qi_value(&token_data.claims)?;

            let mut result_map = crate::new_hashmap();
            result_map.insert(crate::value::MapKey::Keyword(crate::intern::intern_keyword("header")), header);
            result_map.insert(crate::value::MapKey::Keyword(crate::intern::intern_keyword("payload")), payload);

            Ok(Value::Map(result_map))
        }
        Err(e) => Ok(Value::error(e.to_string())),
    }
}

// ========================================
// ヘルパー関数
// ========================================

/// アルゴリズム名をパース
fn parse_algorithm(alg: &str) -> Result<Algorithm, String> {
    match alg.to_uppercase().as_str() {
        "HS256" => Ok(Algorithm::HS256),
        "HS384" => Ok(Algorithm::HS384),
        "HS512" => Ok(Algorithm::HS512),
        "RS256" => Ok(Algorithm::RS256),
        "RS384" => Ok(Algorithm::RS384),
        "RS512" => Ok(Algorithm::RS512),
        "ES256" => Ok(Algorithm::ES256),
        "ES384" => Ok(Algorithm::ES384),
        "PS256" => Ok(Algorithm::PS256),
        "PS384" => Ok(Algorithm::PS384),
        "PS512" => Ok(Algorithm::PS512),
        "EDDSA" => Ok(Algorithm::EdDSA),
        _ => Err(fmt_msg(
            MsgKey::InvalidAlgorithm,
            &["jwt", alg, "HS256, HS384, HS512, RS256, etc."],
        )),
    }
}

/// QiのマップをJSON Valueに変換
fn qi_map_to_json(map: &crate::HashMap<MapKey, Value>) -> Result<JsonValue, String> {
    let mut json_map = serde_json::Map::new();

    for (key, value) in map.iter() {
        // MapKeyを通常の文字列に変換
        let key_str = match key {
            MapKey::Keyword(k) => k.to_string(),
            MapKey::String(s) => {
                // ダブルクォートで囲まれている場合は取り除く
                if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    s[1..s.len() - 1].to_string()
                } else {
                    s.clone()
                }
            }
            MapKey::Symbol(sym) => sym.to_string(),
            MapKey::Integer(i) => i.to_string(),
        };

        let json_value = qi_value_to_json(value)?;
        json_map.insert(key_str, json_value);
    }

    Ok(JsonValue::Object(json_map))
}

/// QiのValueをJSON Valueに変換
fn qi_value_to_json(value: &Value) -> Result<JsonValue, String> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number((*i).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .ok_or_else(|| fmt_msg(MsgKey::InvalidFloat, &["jwt/sign"])),
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::Keyword(s) => Ok(JsonValue::String(s.to_string())),
        Value::Vector(items) | Value::List(items) => {
            let json_items: Result<Vec<JsonValue>, String> =
                items.iter().map(qi_value_to_json).collect();
            Ok(JsonValue::Array(json_items?))
        }
        Value::Map(m) => qi_map_to_json(m),
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["jwt/sign (values)", "primitives, vectors, or maps"],
        )),
    }
}

/// JSON ValueをQiのValueに変換
fn json_to_qi_value(json: &JsonValue) -> Result<Value, String> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Bool(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(fmt_msg(MsgKey::InvalidNumber, &["jwt/verify"]))
            }
        }
        JsonValue::String(s) => Ok(Value::String(s.clone())),
        JsonValue::Array(arr) => {
            let items: Result<Vec<Value>, String> = arr.iter().map(json_to_qi_value).collect();
            Ok(Value::Vector(items?.into()))
        }
        JsonValue::Object(obj) => {
            let mut map = crate::new_hashmap();
            for (k, v) in obj.iter() {
                // JSONキーをキーワード形式（":name"）に変換
                let key = to_map_key(k);
                let value = json_to_qi_value(v)?;
                map.insert(key, value);
            }
            Ok(Value::Map(map))
        }
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category auth/jwt
/// @qi-doc:functions jwt/sign, jwt/verify, jwt/decode
pub const FUNCTIONS: super::NativeFunctions = &[
    ("jwt/sign", native_jwt_sign),
    ("jwt/verify", native_jwt_verify),
    ("jwt/decode", native_jwt_decode),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_sign_and_verify() {
        let mut payload = crate::new_hashmap();
        payload.insert(crate::value::MapKey::Keyword(crate::intern::intern_keyword("user_id")), Value::Integer(123));
        payload.insert(crate::value::MapKey::Keyword(crate::intern::intern_keyword("name")), Value::String("Alice".to_string()));

        let secret = Value::String("my-secret-key".to_string());

        // トークン生成
        let sign_result = native_jwt_sign(&[Value::Map(payload), secret.clone()]).unwrap();

        // 成功時はトークン文字列を直接返す
        let token = match sign_result {
            Value::String(t) => t.clone(),
            _ => panic!("Expected token string, got {:?}", sign_result),
        };

        // トークン検証
        let verify_result = native_jwt_verify(&[Value::String(token), secret]).unwrap();

        // 成功時はペイロードマップを直接返す
        match verify_result {
            Value::Map(payload) => {
                assert_eq!(payload.get(&crate::value::MapKey::Keyword(crate::intern::intern_keyword("user_id"))), Some(&Value::Integer(123)));
            }
            _ => panic!("Expected payload map, got {:?}", verify_result),
        }
    }

    #[test]
    fn test_jwt_decode() {
        let mut payload = crate::new_hashmap();
        payload.insert(crate::value::MapKey::Keyword(crate::intern::intern_keyword("user_id")), Value::Integer(123));

        let secret = Value::String("my-secret-key".to_string());

        // トークン生成
        let sign_result = native_jwt_sign(&[Value::Map(payload), secret]).unwrap();

        // 成功時はトークン文字列を直接返す
        let token = match sign_result {
            Value::String(t) => t.clone(),
            _ => panic!("Expected token string, got {:?}", sign_result),
        };

        // デコード（検証なし）
        let decode_result = native_jwt_decode(&[Value::String(token)]).unwrap();

        // 成功時は {:header ... :payload ...} マップを直接返す
        match decode_result {
            Value::Map(data) => {
                assert!(data.contains_key(&crate::value::MapKey::Keyword(crate::intern::intern_keyword("header"))));
                assert!(data.contains_key(&crate::value::MapKey::Keyword(crate::intern::intern_keyword("payload"))));
            }
            _ => panic!(
                "Expected map with :header and :payload, got {:?}",
                decode_result
            ),
        }
    }

    #[test]
    fn test_jwt_verify_invalid_token() {
        let token = Value::String("invalid.token.here".to_string());
        let secret = Value::String("my-secret-key".to_string());

        let result = native_jwt_verify(&[token, secret]).unwrap();

        match result {
            Value::Map(m) => {
                assert!(m.contains_key(&crate::constants::keywords::error_mapkey()));
            }
            _ => panic!("Expected {{:error ...}}"),
        }
    }
}
