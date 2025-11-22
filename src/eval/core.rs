//! 基本値評価とコア機能
//!
//! リテラル、Symbol、コレクションの評価ロジックを提供します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Env, FStringPart, Value};
use parking_lot::RwLock;
use std::sync::Arc;

use super::helpers::qerr;
use super::Evaluator;

/// マクロ: 引数数チェック
macro_rules! check_args {
    ($args:expr, $expected:expr, $func_name:expr) => {
        if $args.len() != $expected {
            return Err(fmt_msg(
                MsgKey::NeedExactlyNArgs,
                &[$func_name, &$expected.to_string()],
            ));
        }
    };
}

/// list - リストを作成
pub fn native_list(args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(args.iter().cloned().collect()))
}

/// vector - ベクタを作成
pub fn native_vector(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Vector(args.iter().cloned().collect()))
}

/// to-list - List/VectorをListに変換
pub fn native_to_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(qerr(MsgKey::Need1Arg, &["to-list"]));
    }
    match &args[0] {
        Value::List(_) => Ok(args[0].clone()),
        Value::Vector(v) => Ok(Value::List(v.clone())),
        _ => Err(qerr(MsgKey::TypeOnly, &["to-list", "lists or vectors"])),
    }
}

/// to-vector - List/VectorをVectorに変換
pub fn native_to_vector(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(qerr(MsgKey::Need1Arg, &["to-vector"]));
    }
    match &args[0] {
        Value::Vector(_) => Ok(args[0].clone()),
        Value::List(v) => Ok(Value::Vector(v.clone())),
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["to-vector", "lists or vectors"],
        )),
    }
}

/// print - 値を出力
pub fn native_print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

/// number? - 数値かどうか判定
pub fn native_is_number(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "number?");
    Ok(Value::Bool(matches!(
        args[0],
        Value::Integer(_) | Value::Float(_)
    )))
}

/// fn? - 関数かどうか判定
pub fn native_is_fn(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "fn?");
    Ok(Value::Bool(matches!(
        args[0],
        Value::Function(_) | Value::NativeFunc(_)
    )))
}

impl Evaluator {
    /// f-stringを評価
    pub(super) fn eval_fstring(
        &self,
        parts: &[FStringPart],
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        let mut result = String::new();

        for part in parts {
            match part {
                FStringPart::Text(text) => result.push_str(text),
                FStringPart::Code(code) => {
                    // コードをパースして評価
                    let mut parser = crate::parser::Parser::new(code).map_err(|e| {
                        crate::i18n::fmt_msg(crate::i18n::MsgKey::FStringCodeParseError, &[&e])
                    })?;
                    let expr = parser.parse().map_err(|e| {
                        crate::i18n::fmt_msg(crate::i18n::MsgKey::FStringCodeParseError, &[&e])
                    })?;
                    let value = self.eval_with_env(&expr, Arc::clone(&env))?;

                    // 値を文字列に変換
                    let s = match value {
                        Value::String(s) => s.clone(),
                        Value::Integer(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Nil => "nil".to_string(),
                        Value::Keyword(k) => format!(":{}", k),
                        Value::Symbol(s) => s.to_string(),
                        Value::Bytes(b) => {
                            // バイナリデータを16進数表現で表示（最大16バイトまで）
                            let hex: Vec<String> = b
                                .iter()
                                .take(16)
                                .map(|byte| format!("{:02X}", byte))
                                .collect();
                            if b.len() > 16 {
                                format!("#bytes[{} ... ({} bytes)]", hex.join(" "), b.len())
                            } else {
                                format!("#bytes[{}]", hex.join(" "))
                            }
                        }
                        Value::List(items) => {
                            let strs: Vec<_> = items.iter().map(|v| format!("{}", v)).collect();
                            format!("({})", strs.join(" "))
                        }
                        Value::Vector(items) => {
                            let strs: Vec<_> = items.iter().map(|v| format!("{}", v)).collect();
                            format!("[{}]", strs.join(" "))
                        }
                        Value::Map(m) => {
                            let strs: Vec<_> =
                                m.iter().map(|(k, v)| format!(":{} {}", k, v)).collect();
                            format!("{{{}}}", strs.join(" "))
                        }
                        Value::Function(_) => "<function>".to_string(),
                        Value::NativeFunc(nf) => format!("<native-fn:{}>", nf.name),
                        Value::Macro(m) => format!("<macro:{}>", m.name),
                        Value::Atom(a) => format!("<atom:{}>", a.read()),
                        Value::Channel(_) => "<channel>".to_string(),
                        Value::Scope(_) => "<scope>".to_string(),
                        Value::Stream(_) => "<stream>".to_string(),
                        Value::Uvar(id) => format!("<uvar:{}>", id),
                    };
                    result.push_str(&s);
                }
            }
        }

        Ok(Value::String(result))
    }
}
