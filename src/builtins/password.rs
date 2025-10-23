//! パスワードハッシュ機能
//!
//! このモジュールは `auth-password` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// password/hash - パスワードをハッシュ化
///
/// 引数:
/// - password: パスワード文字列
///
/// 戻り値: ハッシュ化されたパスワード文字列
pub fn native_password_hash(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["password/hash"]));
    }

    let password = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["password/hash", "strings"])),
    };

    // Argon2でハッシュ化
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => Ok(Value::String(password_hash.to_string())),
        Err(e) => Err(fmt_msg(
            MsgKey::PasswordHashError,
            &["password/hash", &e.to_string()],
        )),
    }
}

/// password/verify - パスワードを検証
///
/// 引数:
/// - password: パスワード文字列
/// - hash: ハッシュ文字列
///
/// 戻り値: 検証結果（true/false）
pub fn native_password_verify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["password/verify"]));
    }

    let password = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["password/verify (password)", "strings"],
            ))
        }
    };

    let hash_str = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["password/verify (hash)", "strings"],
            ))
        }
    };

    // ハッシュをパース
    let parsed_hash = match PasswordHash::new(hash_str) {
        Ok(h) => h,
        Err(_) => return Ok(Value::Bool(false)), // 不正なハッシュ形式の場合はfalse
    };

    // パスワードを検証
    let argon2 = Argon2::default();
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(Value::Bool(is_valid))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category auth/password
/// @qi-doc:functions password/hash, password/verify
pub const FUNCTIONS: super::NativeFunctions = &[
    ("password/hash", native_password_hash),
    ("password/verify", native_password_verify),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = Value::String("my-secure-password".to_string());

        // パスワードをハッシュ化
        let hash_result = native_password_hash(&[password.clone()]).unwrap();

        let hash = match hash_result {
            Value::String(h) => h,
            _ => panic!("Expected string hash"),
        };

        // 正しいパスワードで検証
        let verify_result =
            native_password_verify(&[password.clone(), Value::String(hash.clone())]).unwrap();
        assert_eq!(verify_result, Value::Bool(true));

        // 間違ったパスワードで検証
        let wrong_password = Value::String("wrong-password".to_string());
        let verify_result = native_password_verify(&[wrong_password, Value::String(hash)]).unwrap();
        assert_eq!(verify_result, Value::Bool(false));
    }

    #[test]
    fn test_password_verify_invalid_hash() {
        let password = Value::String("test".to_string());
        let invalid_hash = Value::String("invalid-hash".to_string());

        let result = native_password_verify(&[password, invalid_hash]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let password1 = Value::String("password1".to_string());
        let password2 = Value::String("password2".to_string());

        let hash1 = native_password_hash(&[password1]).unwrap();
        let hash2 = native_password_hash(&[password2]).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        // ソルトが異なるため、同じパスワードでも異なるハッシュが生成される
        let password = Value::String("test-password".to_string());

        let hash1 = native_password_hash(&[password.clone()]).unwrap();
        let hash2 = native_password_hash(&[password]).unwrap();

        assert_ne!(hash1, hash2);
    }
}
