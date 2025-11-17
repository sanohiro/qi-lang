//! レスポンス生成関数

use super::helpers::kw;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

pub fn native_server_ok(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/ok", "1"]));
    }

    let body = match &args[0] {
        Value::String(s) => s.clone(),
        v => format!("{}", v),
    };

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(200));
    resp.insert(kw("body"), Value::String(body));

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/json - JSONレスポンスを作成
pub fn native_server_json(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/json", "1"]));
    }

    // データをJSON文字列に変換
    let json_result = crate::builtins::json::native_stringify(&[args[0].clone()])?;
    let json_str = match json_result {
        Value::String(s) => s,
        Value::Map(m) if m.contains_key(":error") => {
            return Err(fmt_msg(MsgKey::JsonStringifyError, &[]));
        }
        _ => return Err(fmt_msg(MsgKey::JsonStringifyError, &[])),
    };

    let status = if args.len() > 1 {
        if let Value::Map(opts) = &args[1] {
            let status_key = kw("status");
            match opts.get(&status_key) {
                Some(Value::Integer(s)) => *s,
                _ => 200,
            }
        } else {
            200
        }
    } else {
        200
    };

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(status));
    resp.insert(kw("body"), Value::String(json_str));

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("application/json; charset=utf-8".to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/response - 汎用HTTPレスポンスを作成
/// (server/response status body)
/// status: HTTPステータスコード (Integer)
/// body: レスポンスボディ (String)
pub fn native_server_response(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/response", "2"]));
    }

    let status = match &args[0] {
        Value::Integer(n) => *n,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["server/response", "an integer"],
            ))
        }
    };

    let body = match &args[1] {
        Value::String(s) => s.clone(),
        v => format!("{}", v),
    };

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(status));
    resp.insert(kw("body"), Value::String(body));

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/not-found - 404レスポンスを作成
pub fn native_server_not_found(args: &[Value]) -> Result<Value, String> {
    let body = if args.is_empty() {
        "Not Found".to_string()
    } else {
        match &args[0] {
            Value::String(s) => s.clone(),
            v => format!("{}", v),
        }
    };

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(404));
    resp.insert(kw("body"), Value::String(body));

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/no-content - 204 No Contentレスポンスを作成
pub fn native_server_no_content(_args: &[Value]) -> Result<Value, String> {
    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(204));
    resp.insert(kw("body"), Value::String(String::new()));
    Ok(Value::Map(resp))
}

