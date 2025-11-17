use super::*;

/// HTTP GETリクエスト（詳細版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/get! "https://api.example.com")  ;=> {:status 200 :body "..."}
/// 例: (http/get! "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_get_bang(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get!", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("GET", url, None, headers.as_ref(), timeout)
}

/// HTTP POSTリクエスト（詳細版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/post! "https://api.example.com" {:key "value"})
/// 例: (http/post! "https://api.example.com" {:key "value"} {:headers {"Authorization" "Bearer token"}})
pub fn native_post_bang(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/post!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post!", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("POST", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP PUTリクエスト（詳細版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/put! "https://api.example.com/1" {:key "value"})
/// 例: (http/put! "https://api.example.com/1" {:key "value"} {:timeout 10000})
pub fn native_put_bang(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/put!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/put!", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("PUT", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP DELETEリクエスト（詳細版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/delete! "https://api.example.com/1")
/// 例: (http/delete! "https://api.example.com/1" {:headers {"Authorization" "Bearer token"}})
pub fn native_delete_bang(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/delete!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/delete!", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("DELETE", url, None, headers.as_ref(), timeout)
}

/// HTTP PATCHリクエスト（詳細版）
/// 引数: URL文字列、ボディデータ、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/patch! "https://api.example.com/1" {:key "value"})
/// 例: (http/patch! "https://api.example.com/1" {:key "value"} {:timeout 10000})
pub fn native_patch_bang(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/patch!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/patch!", "URL"])),
    };

    let opts = args.get(2);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("PATCH", url, Some(&args[1]), headers.as_ref(), timeout)
}

/// HTTP HEADリクエスト（詳細版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body ""}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/head! "https://api.example.com")
/// 例: (http/head! "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_head_bang(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/head!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/head!", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("HEAD", url, None, headers.as_ref(), timeout)
}

/// HTTP OPTIONSリクエスト（詳細版）
/// 引数: URL文字列、オプション（省略可）
/// 戻り値: {:status 200 :headers {...} :body "..."}
/// エラー時: {:error {:type "timeout" :message "..."}}
/// 例: (http/options! "https://api.example.com")
/// 例: (http/options! "https://api.example.com" {:headers {"Authorization" "Bearer token"}})
pub fn native_options_bang(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/options!"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/options!", "URL"])),
    };

    let opts = args.get(1);
    let (headers, timeout) = helpers::parse_http_options(opts)?;

    core::http_request_detailed("OPTIONS", url, None, headers.as_ref(), timeout)
}
