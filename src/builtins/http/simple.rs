use super::*;

/// HTTP GETリクエスト（シンプル版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/get "https://api.example.com")  ;=> "{"data": "..."}"
/// 例: (http/get "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("GET", url, None, headers.as_ref(), timeout)
}

/// HTTP POSTリクエスト（シンプル版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/post "https://api.example.com" {:key "value"})
/// 例: (http/post "https://api.example.com" {:key "value"} {:headers {"Authorization" "Bearer token"}})
pub fn native_post(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/post"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("POST", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP PUTリクエスト（シンプル版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/put "https://api.example.com/1" {:key "value"})
/// 例: (http/put "https://api.example.com/1" {:key "value"} {:timeout 10000})
pub fn native_put(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/put"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/put", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("PUT", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP DELETEリクエスト（シンプル版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/delete "https://api.example.com/1")
/// 例: (http/delete "https://api.example.com/1" {:headers {"Authorization" "Bearer token"}})
pub fn native_delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/delete"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/delete", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("DELETE", url, None, headers.as_ref(), timeout)
}

/// HTTP PATCHリクエスト（シンプル版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/patch "https://api.example.com/1" {:key "value"})
/// 例: (http/patch "https://api.example.com/1" {:key "value"} {:timeout 10000})
pub fn native_patch(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/patch"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/patch", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("PATCH", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP HEADリクエスト（シンプル版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列、通常は空）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/head "https://api.example.com")
/// 例: (http/head "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_head(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/head"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/head", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("HEAD", url, None, headers.as_ref(), timeout)
}

/// HTTP OPTIONSリクエスト（シンプル版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: レスポンスボディ（文字列）
/// エラー時: Err(エラーメッセージ)
/// 例: (http/options "https://api.example.com")
/// 例: (http/options "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_options(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/options"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/options", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request("OPTIONS", url, None, headers.as_ref(), timeout)
}
