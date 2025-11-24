use super::*;

/// HTTP GET（ストリーミング版）- レスポンスボディを行ごとに遅延読み込み
/// 引数: (http/get-stream "url") - テキストモード（行ごと）
///      (http/get-stream "url" :bytes) - バイナリモード（バイトチャンクごと）
pub fn native_get_stream(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get-stream"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get-stream", "URL"])),
    };

    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if &**k == "bytes");

    core::http_stream("GET", &url, None, None, None, None, None, is_bytes)
}

/// HTTP POST（ストリーミング版）- レスポンスボディを行ごとに遅延読み込み
/// 引数: (http/post-stream "url" body) - テキストモード
///      (http/post-stream "url" body :bytes) - バイナリモード
pub fn native_post_stream(args: &[Value]) -> Result<Value, String> {
    crate::check_args!(args, 2, "http/post-stream");

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post-stream", "URL"])),
    };

    let is_bytes = args.len() >= 3 && matches!(&args[2], Value::Keyword(k) if &**k == "bytes");

    core::http_stream(
        "POST",
        &url,
        Some(&args[1]),
        None,
        None,
        None,
        None,
        is_bytes,
    )
}

/// HTTP Request（ストリーミング版）- 詳細な設定でストリーミング受信
/// 引数: (http/request-stream {:method "GET" :url "..."}) - テキストモード
///      (http/request-stream {:method "GET" :url "..."} :bytes) - バイナリモード
pub fn native_request_stream(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/request-stream"]));
    }

    let config = match &args[0] {
        Value::Map(m) => m,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeMap,
                &["http/request-stream", "argument"],
            ))
        }
    };

    // キーワードキーを生成（静的キーワードの初期化、CONTRIBUTING.md参照）
    #[allow(clippy::expect_used)]
    let method_key = Value::Keyword(crate::intern::intern_keyword("method"))
        .to_map_key()
        .expect("method keyword should be valid");
    #[allow(clippy::expect_used)]
    let url_key = Value::Keyword(crate::intern::intern_keyword("url"))
        .to_map_key()
        .expect("url keyword should be valid");
    #[allow(clippy::expect_used)]
    let body_key = Value::Keyword(crate::intern::intern_keyword("body"))
        .to_map_key()
        .expect("body keyword should be valid");
    #[allow(clippy::expect_used)]
    let headers_key = Value::Keyword(crate::intern::intern_keyword("headers"))
        .to_map_key()
        .expect("headers keyword should be valid");
    #[allow(clippy::expect_used)]
    let timeout_key = Value::Keyword(crate::intern::intern_keyword("timeout"))
        .to_map_key()
        .expect("timeout keyword should be valid");
    #[allow(clippy::expect_used)]
    let basic_auth_key = Value::Keyword(crate::intern::intern_keyword("basic-auth"))
        .to_map_key()
        .expect("basic-auth keyword should be valid");
    #[allow(clippy::expect_used)]
    let bearer_token_key = Value::Keyword(crate::intern::intern_keyword("bearer-token"))
        .to_map_key()
        .expect("bearer-token keyword should be valid");

    // メソッドを取得（文字列またはキーワード）し、大文字に正規化
    let method = match config.get(&method_key) {
        Some(Value::String(s)) => s.to_uppercase(),
        Some(Value::Keyword(k)) => k.to_uppercase(),
        _ => "GET".to_string(),
    };

    let url = match config.get(&url_key) {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::KeyNotFound, &["url"])),
    };

    let body = config.get(&body_key);

    // 追加オプションを取得
    let headers = config.get(&headers_key).and_then(|v| match v {
        Value::Map(m) => Some(m),
        _ => None,
    });

    let timeout_ms = config.get(&timeout_key).and_then(|v| match v {
        Value::Integer(i) => Some(*i as u64),
        _ => None,
    });

    let basic_auth = config.get(&basic_auth_key).and_then(|v| match v {
        Value::Vector(v) if v.len() == 2 => {
            let user = match &v[0] {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }?;
            let pass = match &v[1] {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }?;
            Some((user, pass))
        }
        _ => None,
    });

    let bearer_token = config.get(&bearer_token_key).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        _ => None,
    });

    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if &**k == "bytes");

    core::http_stream(
        &method,
        &url,
        body,
        headers,
        timeout_ms,
        basic_auth,
        bearer_token,
        is_bytes,
    )
}
