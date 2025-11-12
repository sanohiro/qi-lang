//! HTTPクライアントモジュール
//!
//! HTTP通信関数を提供:
//! - get/post/put/delete/patch/head/options: 各HTTPメソッド
//! - request: 詳細なリクエスト設定
//! - get-async/post-async: 非同期版
//! - get-stream/post-stream/request-stream: ストリーミング版
//!
//! このモジュールは `http-client` feature でコンパイルされます。

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use crossbeam_channel::bounded;
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::RwLock;
use reqwest::blocking::Client;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

/// gzip圧縮ヘルパー関数
fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// オプションMapからヘッダーとタイムアウトを抽出する
/// 引数: オプションMap（Option<&Value>）
/// 戻り値: (ヘッダーMap, タイムアウトms)
fn parse_http_options(
    opts: Option<&Value>,
) -> Result<(Option<crate::HashMap<String, Value>>, u64), String> {
    let Some(Value::Map(opts_map)) = opts else {
        // オプションがない場合はデフォルト値
        return Ok((None, 30000));
    };

    // キーを準備
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

    // ヘッダーを取得
    let mut headers = opts_map
        .get(&headers_key)
        .and_then(|v| match v {
            Value::Map(m) => Some(m.clone()),
            _ => None,
        })
        .unwrap_or_default();

    // Basic Auth処理
    if let Some(Value::Vector(v)) = opts_map.get(&basic_auth_key) {
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
    if let Some(Value::String(token)) = opts_map.get(&bearer_token_key) {
        headers.insert(
            "authorization".to_string(),
            Value::String(format!("Bearer {}", token)),
        );
    }

    // タイムアウトを取得
    let timeout = opts_map
        .get(&timeout_key)
        .and_then(|v| match v {
            Value::Integer(i) if *i > 0 => Some(*i as u64),
            _ => None,
        })
        .unwrap_or(30000);

    let headers_opt = if headers.is_empty() {
        None
    } else {
        Some(headers)
    };

    Ok((headers_opt, timeout))
}

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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("GET", url, None, headers.as_ref(), timeout)
}

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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("GET", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("POST", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("POST", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("PUT", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("PUT", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("DELETE", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("DELETE", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("PATCH", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("PATCH", url, Some(&args[1]), headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("HEAD", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("HEAD", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request("OPTIONS", url, None, headers.as_ref(), timeout)
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
    let (headers, timeout) = parse_http_options(opts)?;

    http_request_detailed("OPTIONS", url, None, headers.as_ref(), timeout)
}

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

    http_request_detailed(method, url, body, headers_ref, timeout)
}

/// HTTP GETリクエスト (非同期)
pub fn native_get_async(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get-async"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get-async", "URL"])),
    };

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: sender.clone(),
        receiver,
    });

    rayon::spawn(move || {
        let result = native_get(&[Value::String(url)]);
        let _ = sender.send(result.unwrap_or_else(Value::String));
    });

    Ok(Value::Channel(result_channel))
}

/// HTTP POSTリクエスト (非同期)
pub fn native_post_async(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "http/post-async");

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post-async", "URL"])),
    };

    let body = args[1].clone();

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: sender.clone(),
        receiver,
    });

    rayon::spawn(move || {
        let result = native_post(&[Value::String(url), body]);
        let _ = sender.send(result.unwrap_or_else(Value::String));
    });

    Ok(Value::Channel(result_channel))
}

/// HTTPリクエストの実装（シンプル版：bodyの文字列のみ返す）
fn http_request(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: Option<&crate::HashMap<String, Value>>,
    timeout_ms: u64,
) -> Result<Value, String> {
    // 詳細版を呼び出す
    let result = http_request_detailed(method, url, body, headers, timeout_ms)?;

    // 詳細版の戻り値を処理
    match result {
        Value::Map(m) => {
            // キーを準備
            let status_key = Value::Keyword("status".to_string())
                .to_map_key()
                .expect("status keyword should be valid");
            let body_key = Value::Keyword("body".to_string())
                .to_map_key()
                .expect("body keyword should be valid");
            let error_key = Value::Keyword("error".to_string())
                .to_map_key()
                .expect("error keyword should be valid");

            // errorキーがある場合（ネットワークエラー等）
            if let Some(Value::Map(err_map)) = m.get(&error_key) {
                let message_key = Value::Keyword("message".to_string())
                    .to_map_key()
                    .expect("message keyword should be valid");

                if let Some(Value::String(msg)) = err_map.get(&message_key) {
                    return Err(msg.clone());
                }
                return Err("Unexpected error format".to_string());
            }

            // statusキーとbodyキーを取得
            let status = m
                .get(&status_key)
                .and_then(|v| match v {
                    Value::Integer(i) => Some(*i),
                    _ => None,
                })
                .ok_or_else(|| "Missing status in response".to_string())?;

            let body_val = m
                .get(&body_key)
                .ok_or_else(|| "Missing body in response".to_string())?;

            // ステータスコードをチェック（2xx = 200-299）
            if (200..300).contains(&status) {
                // 成功：bodyを返す
                Ok(body_val.clone())
            } else {
                // 失敗：エラーメッセージを生成
                // bodyが文字列の場合は含める（長い場合は切り詰め）
                let body_preview = match body_val {
                    Value::String(s) => {
                        if s.len() > 200 {
                            format!("{}...", &s[..200])
                        } else {
                            s.clone()
                        }
                    }
                    _ => String::new(),
                };

                if body_preview.is_empty() {
                    Err(fmt_msg(MsgKey::HttpErrorStatus, &[&status.to_string()]))
                } else {
                    Err(format!("HTTP error {}: {}", status, body_preview))
                }
            }
        }
        _ => Err("Unexpected response format".to_string()),
    }
}

/// HTTPリクエストの実装（詳細版：Map形式で詳細情報を返す）
fn http_request_detailed(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: Option<&crate::HashMap<String, Value>>,
    timeout_ms: u64,
) -> Result<Value, String> {
    // デフォルトタイムアウト（30秒）の場合は共有Clientを使用
    // カスタムタイムアウトの場合のみ新しいClientを作成
    let custom_client;
    let client = if timeout_ms == 30000 {
        &crate::builtins::lazy_init::http_client::CLIENT
    } else {
        custom_client = Client::builder()
            .gzip(true)
            .deflate(true)
            .brotli(true)
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|e| fmt_msg(MsgKey::HttpClientError, &[&e.to_string()]))?;
        &custom_client
    };

    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        "HEAD" => client.head(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        _ => return Err(fmt_msg(MsgKey::HttpUnsupportedMethod, &[method])),
    };

    // 圧縮が必要かチェック
    let should_compress = headers
        .and_then(|h| h.get("content-encoding"))
        .and_then(|v| match v {
            Value::String(s) => Some(s.to_lowercase()),
            _ => None,
        })
        .map(|s| s == "gzip")
        .unwrap_or(false);

    // ヘッダー追加
    if let Some(h) = headers {
        for (k, v) in h.iter() {
            if let Value::String(val) = v {
                // Qiパーサーがキーにダブルクォートを含める場合があるため除去
                let key = k.trim_matches('"');
                let value = val.trim_matches('"');
                request = request.header(key, value);
            }
        }
    }

    // ボディ追加
    if let Some(b) = body {
        match b {
            Value::String(s) => {
                if should_compress {
                    // gzip圧縮して送信
                    let compressed = compress_gzip(s.as_bytes())
                        .map_err(|e| fmt_msg(MsgKey::HttpCompressionError, &[&e.to_string()]))?;
                    request = request.header("Content-Encoding", "gzip").body(compressed);
                } else {
                    request = request.body(s.clone());
                }
            }
            _ => {
                // JSON自動変換
                let json_str = crate::builtins::json::native_stringify(std::slice::from_ref(b))?;
                // 新仕様: json/stringifyは値を直接返す（{:ok}ラップなし）
                if let Value::String(s) = json_str {
                    if should_compress {
                        // JSON を圧縮して送信
                        let compressed = compress_gzip(s.as_bytes()).map_err(|e| {
                            fmt_msg(MsgKey::HttpCompressionError, &[&e.to_string()])
                        })?;
                        request = request
                            .header("Content-Type", "application/json")
                            .header("Content-Encoding", "gzip")
                            .body(compressed);
                    } else {
                        request = request
                            .header("Content-Type", "application/json")
                            .body(s.clone());
                    }
                }
            }
        }
    }

    // リクエスト送信
    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16() as i64;

            // ヘッダーを取得
            let headers: crate::HashMap<String, Value> = response
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        Value::String(v.to_str().unwrap_or("").to_string()),
                    )
                })
                .collect();

            // ボディを取得
            let body = response.text().unwrap_or_else(|_| String::new());

            // キーワードキーを生成
            let status_key = Value::Keyword("status".to_string())
                .to_map_key()
                .expect("status keyword should be valid");
            let headers_key = Value::Keyword("headers".to_string())
                .to_map_key()
                .expect("headers keyword should be valid");
            let body_key = Value::Keyword("body".to_string())
                .to_map_key()
                .expect("body keyword should be valid");

            Ok(Value::Map(
                [
                    (status_key, Value::Integer(status)),
                    (headers_key, Value::Map(headers)),
                    (body_key, Value::String(body)),
                ]
                .into_iter()
                .collect(),
            ))
        }
        Err(e) => {
            let error_type = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "network"
            } else {
                "unknown"
            };

            // エラーレスポンスもキーワードキーに変更
            let error_key = Value::Keyword("error".to_string())
                .to_map_key()
                .expect("error keyword should be valid");
            let type_key = Value::Keyword("type".to_string())
                .to_map_key()
                .expect("type keyword should be valid");
            let message_key = Value::Keyword("message".to_string())
                .to_map_key()
                .expect("message keyword should be valid");

            Ok(Value::Map(
                [(
                    error_key,
                    Value::Map(
                        [
                            (type_key, Value::String(error_type.to_string())),
                            (message_key, Value::String(e.to_string())),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )]
                .into_iter()
                .collect(),
            ))
        }
    }
}

// ========================================
// ストリーミング版 HTTP関数
// ========================================

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

    http_stream("GET", &url, None, is_bytes)
}

/// HTTP POST（ストリーミング版）- レスポンスボディを行ごとに遅延読み込み
/// 引数: (http/post-stream "url" body) - テキストモード
///      (http/post-stream "url" body :bytes) - バイナリモード
pub fn native_post_stream(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "http/post-stream");

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post-stream", "URL"])),
    };

    let is_bytes = args.len() >= 3 && matches!(&args[2], Value::Keyword(k) if k == "bytes");

    http_stream("POST", &url, Some(&args[1]), is_bytes)
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

    http_stream(method, &url, body, is_bytes)
}

/// HTTPストリーミングの共通実装（真のストリーミング - メモリに全体を読み込まない）
fn http_stream(
    method: &str,
    url: &str,
    body: Option<&Value>,
    is_bytes: bool,
) -> Result<Value, String> {
    // 共有Clientを使用（デフォルトで30秒タイムアウト設定済み）
    let client = &crate::builtins::lazy_init::http_client::CLIENT;

    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => return Err(fmt_msg(MsgKey::HttpUnsupportedMethod, &[method])),
    };

    // ボディ追加
    if let Some(b) = body {
        match b {
            Value::String(s) => {
                request = request.body(s.clone());
            }
            _ => {
                // JSON自動変換
                let json_str = crate::builtins::json::native_stringify(std::slice::from_ref(b))?;
                // 新仕様: json/stringifyは値を直接返す（{:ok}ラップなし）
                if let Value::String(s) = json_str {
                    request = request
                        .header("Content-Type", "application/json")
                        .body(s.clone());
                }
            }
        }
    }

    // リクエスト送信
    let response = request
        .send()
        .map_err(|e| fmt_msg(MsgKey::HttpStreamRequestFailed, &[&e.to_string()]))?;

    if !response.status().is_success() {
        return Err(fmt_msg(
            MsgKey::HttpStreamError,
            &[&response.status().to_string()],
        ));
    }

    if is_bytes {
        // バイナリモード：チャンクごとに遅延読み込み
        use std::io::Read;

        const CHUNK_SIZE: usize = 4096;
        let response = Arc::new(parking_lot::Mutex::new(response));

        let stream = Stream {
            next_fn: Box::new(move || {
                let mut resp = response.lock();
                let mut buffer = vec![0u8; CHUNK_SIZE];

                match resp.read(&mut buffer) {
                    Ok(0) => None, // EOF
                    Ok(n) => {
                        buffer.truncate(n);
                        // バイト配列をIntegerのVectorに変換
                        let bytes: im::Vector<Value> =
                            buffer.iter().map(|&b| Value::Integer(b as i64)).collect();
                        Some(Value::Vector(bytes))
                    }
                    Err(_) => None, // エラー時はストリーム終了
                }
            }),
        };

        Ok(Value::Stream(Arc::new(RwLock::new(stream))))
    } else {
        // テキストモード：行ごとに遅延読み込み
        use std::io::BufRead;

        let reader = std::io::BufReader::new(response);
        let lines = Arc::new(parking_lot::Mutex::new(reader.lines()));

        let stream = Stream {
            next_fn: Box::new(move || {
                let mut lines_iter = lines.lock();
                lines_iter
                    .next()
                    .and_then(|result| result.ok().map(Value::String))
            }),
        };

        Ok(Value::Stream(Arc::new(RwLock::new(stream))))
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category net/http
/// @qi-doc:functions get, post, put, delete, patch, head, options, request, get!, post!, put!, delete!, patch!, head!, options!, get-stream, post-stream, request-stream
pub const FUNCTIONS: super::NativeFunctions = &[
    // シンプル版（bodyのみ返す）
    ("http/get", native_get),
    ("http/post", native_post),
    ("http/put", native_put),
    ("http/delete", native_delete),
    ("http/patch", native_patch),
    ("http/head", native_head),
    ("http/options", native_options),
    // 詳細版（!付き、Map形式で詳細情報を返す）
    ("http/get!", native_get_bang),
    ("http/post!", native_post_bang),
    ("http/put!", native_put_bang),
    ("http/delete!", native_delete_bang),
    ("http/patch!", native_patch_bang),
    ("http/head!", native_head_bang),
    ("http/options!", native_options_bang),
    // 詳細制御版（元からの仕様）
    ("http/request", native_request),
    // 非同期版
    ("http/get-async", native_get_async),
    ("http/post-async", native_post_async),
    // ストリーミング版
    ("http/get-stream", native_get_stream),
    ("http/post-stream", native_post_stream),
    ("http/request-stream", native_request_stream),
];
