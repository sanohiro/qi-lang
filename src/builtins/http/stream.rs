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

    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if k == "bytes");

    core::http_stream("GET", &url, None, is_bytes)
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

    let is_bytes = args.len() >= 3 && matches!(&args[2], Value::Keyword(k) if k == "bytes");

    core::http_stream("POST", &url, Some(&args[1]), is_bytes)
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

    // キーワードキーを生成
    let method_key = Value::Keyword("method".to_string())
        .to_map_key()
        .expect("method keyword should be valid");
    let url_key = Value::Keyword("url".to_string())
        .to_map_key()
        .expect("url keyword should be valid");
    let body_key = Value::Keyword("body".to_string())
        .to_map_key()
        .expect("body keyword should be valid");

    let method = match config.get(&method_key) {
        Some(Value::String(s)) => s.as_str(),
        _ => "GET",
    };

    let url = match config.get(&url_key) {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::KeyNotFound, &["url"])),
    };

    let body = config.get(&body_key);

    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if k == "bytes");

    core::http_stream(method, &url, body, is_bytes)
}
