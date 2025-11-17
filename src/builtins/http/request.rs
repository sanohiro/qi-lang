use super::*;

/// 詳細なHTTPリクエスト
pub fn native_request(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/request"]));
    }

    let opts = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(fmt_msg(MsgKey::MustBeMap, &["http/request", "argument"])),
    };

    // オプションをパース（キーワードキーに対応）
    let method_key = Value::Keyword("method".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "method".to_string());
    let url_key = Value::Keyword("url".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "url".to_string());
    let body_key = Value::Keyword("body".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "body".to_string());
    let headers_key = Value::Keyword("headers".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "headers".to_string());
    let basic_auth_key = Value::Keyword("basic-auth".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "basic-auth".to_string());
    let bearer_token_key = Value::Keyword("bearer-token".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "bearer-token".to_string());
    let timeout_key = Value::Keyword("timeout".to_string())
        .to_map_key()
        .unwrap_or_else(|_| "timeout".to_string());

    let method = opts
        .get(&method_key)
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            Value::Keyword(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("GET");

    let url = opts
        .get(&url_key)
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::HttpRequestUrlRequired, &[]))?;

    let body = opts.get(&body_key);

    let mut headers = opts
        .get(&headers_key)
        .and_then(|v| match v {
            Value::Map(m) => Some(m.clone()),
            _ => None,
        })
        .unwrap_or_default();

    // Basic Auth処理
    if let Some(Value::Vector(v)) = opts.get(&basic_auth_key) {
        if v.len() == 2 {
            if let (Value::String(user), Value::String(pass)) = (&v[0], &v[1]) {
                use base64::{engine::general_purpose, Engine as _};
                let credentials = format!("{}:{}", user, pass);
                let encoded = general_purpose::STANDARD.encode(credentials);
                headers.insert(
                    "authorization".to_string(),
                    Value::String(format!("Basic {}", encoded)),
                );
            }
        }
    }

    // Bearer Token処理
    if let Some(Value::String(token)) = opts.get(&bearer_token_key) {
        headers.insert(
            "authorization".to_string(),
            Value::String(format!("Bearer {}", token)),
        );
    }

    let headers_ref = if headers.is_empty() {
        None
    } else {
        Some(&headers)
    };

    let timeout = opts
        .get(&timeout_key)
        .and_then(|v| match v {
            Value::Integer(i) if *i > 0 => Some(*i as u64),
            _ => None,
        })
        .unwrap_or(30000);

    core::http_request_detailed(method, url, body, headers_ref, timeout)
}
