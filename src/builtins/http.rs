//! HTTPクライアントモジュール
//!
//! HTTP通信関数を提供:
//! - get/post/put/delete/patch/head/options: 各HTTPメソッド
//! - request: 詳細なリクエスト設定
//! - get-async/post-async: 非同期版
//! - get-stream/post-stream/request-stream: ストリーミング版

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use crossbeam_channel::bounded;
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::RwLock;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

/// gzip圧縮ヘルパー関数
fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// HTTP GETリクエスト
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get", "URL"])),
    };

    http_request("GET", url, None, None, 30000)
}

/// HTTP POSTリクエスト
pub fn native_post(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/post"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post", "URL"])),
    };

    http_request("POST", url, Some(&args[1]), None, 30000)
}

/// HTTP PUTリクエスト
pub fn native_put(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/put"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/put", "URL"])),
    };

    http_request("PUT", url, Some(&args[1]), None, 30000)
}

/// HTTP DELETEリクエスト
pub fn native_delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/delete"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/delete", "URL"])),
    };

    http_request("DELETE", url, None, None, 30000)
}

/// HTTP PATCHリクエスト
pub fn native_patch(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/patch"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/patch", "URL"])),
    };

    http_request("PATCH", url, Some(&args[1]), None, 30000)
}

/// HTTP HEADリクエスト
pub fn native_head(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/head"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/head", "URL"])),
    };

    http_request("HEAD", url, None, None, 30000)
}

/// HTTP OPTIONSリクエスト
pub fn native_options(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/options"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/options", "URL"])),
    };

    http_request("OPTIONS", url, None, None, 30000)
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

    // オプションをパース
    let method = opts
        .get("method")
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            Value::Keyword(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("GET");

    let url = opts
        .get("url")
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| fmt_msg(MsgKey::HttpRequestUrlRequired, &[]))?;

    let body = opts.get("body");

    let mut headers = opts.get("headers")
        .and_then(|v| match v {
            Value::Map(m) => Some(m.clone()),
            _ => None,
        })
        .unwrap_or_default();

    // Basic Auth処理
    if let Some(basic_auth) = opts.get("basic-auth") {
        if let Value::Vector(v) = basic_auth {
            if v.len() == 2 {
                if let (Value::String(user), Value::String(pass)) = (&v[0], &v[1]) {
                    use base64::{Engine as _, engine::general_purpose};
                    let credentials = format!("{}:{}", user, pass);
                    let encoded = general_purpose::STANDARD.encode(credentials);
                    headers.insert("authorization".to_string(), Value::String(format!("Basic {}", encoded)));
                }
            }
        }
    }

    // Bearer Token処理
    if let Some(bearer) = opts.get("bearer-token") {
        if let Value::String(token) = bearer {
            headers.insert("authorization".to_string(), Value::String(format!("Bearer {}", token)));
        }
    }

    let headers_ref = if headers.is_empty() { None } else { Some(&headers) };

    let timeout = opts
        .get("timeout")
        .and_then(|v| match v {
            Value::Integer(i) if *i > 0 => Some(*i as u64),
            _ => None,
        })
        .unwrap_or(30000);

    http_request(method, url, body, headers_ref, timeout)
}

/// HTTP GETリクエスト (非同期)
pub fn native_get_async(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
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

    std::thread::spawn(move || {
        let result = native_get(&[Value::String(url)]);
        let _ = sender.send(result.unwrap_or_else(Value::String));
    });

    Ok(Value::Channel(result_channel))
}

/// HTTP POSTリクエスト (非同期)
pub fn native_post_async(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/post-async"]));
    }

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

    std::thread::spawn(move || {
        let result = native_post(&[Value::String(url), body]);
        let _ = sender.send(result.unwrap_or_else(Value::String));
    });

    Ok(Value::Channel(result_channel))
}

/// HTTPリクエストの実装
fn http_request(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: Option<&HashMap<String, Value>>,
    timeout_ms: u64,
) -> Result<Value, String> {
    let client = Client::builder()
        .gzip(true)      // gzip自動解凍を有効化
        .deflate(true)   // deflate自動解凍を有効化
        .brotli(true)    // brotli自動解凍を有効化
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| fmt_msg(MsgKey::HttpClientError, &[&e.to_string()]))?;

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
        for (k, v) in h {
            if let Value::String(val) = v {
                request = request.header(k.as_str(), val.as_str());
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
                    request = request.body(compressed);
                } else {
                    request = request.body(s.clone());
                }
            }
            _ => {
                // JSON自動変換
                let json_str = crate::builtins::json::native_stringify(std::slice::from_ref(b))?;
                if let Value::Map(m) = json_str {
                    if let Some(Value::String(s)) = m.get("ok") {
                        if should_compress {
                            // JSON を圧縮して送信
                            let compressed = compress_gzip(s.as_bytes())
                                .map_err(|e| fmt_msg(MsgKey::HttpCompressionError, &[&e.to_string()]))?;
                            request = request
                                .header("Content-Type", "application/json")
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
    }

    // リクエスト送信
    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16() as i64;

            // ヘッダーを取得
            let headers: HashMap<String, Value> = response
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

            Ok(Value::Map(
                [(
                    "ok".to_string(),
                    Value::Map(
                        [
                            ("status".to_string(), Value::Integer(status)),
                            ("headers".to_string(), Value::Map(headers)),
                            ("body".to_string(), Value::String(body)),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )]
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

            Ok(Value::Map(
                [(
                    "error".to_string(),
                    Value::Map(
                        [
                            ("type".to_string(), Value::String(error_type.to_string())),
                            ("message".to_string(), Value::String(e.to_string())),
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
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["http/post-stream"]));
    }

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
        _ => return Err(fmt_msg(MsgKey::MustBeMap, &["http/request-stream", "argument"])),
    };

    let method = match config.get("method") {
        Some(Value::String(s)) => s.as_str(),
        _ => "GET",
    };

    let url = match config.get("url") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::KeyNotFound, &["url"])),
    };

    let body = config.get("body");

    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if k == "bytes");

    http_stream(method, &url, body, is_bytes)
}

/// HTTPストリーミングの共通実装
fn http_stream(method: &str, url: &str, body: Option<&Value>, is_bytes: bool) -> Result<Value, String> {
    let client = Client::builder()
        .gzip(true)      // gzip自動解凍を有効化
        .deflate(true)   // deflate自動解凍を有効化
        .brotli(true)    // brotli自動解凍を有効化
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| fmt_msg(MsgKey::HttpStreamClientError, &[&e.to_string()]))?;

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
                if let Value::Map(m) = json_str {
                    if let Some(Value::String(s)) = m.get("ok") {
                        request = request
                            .header("Content-Type", "application/json")
                            .body(s.clone());
                    }
                }
            }
        }
    }

    // リクエスト送信
    let response = request
        .send()
        .map_err(|e| fmt_msg(MsgKey::HttpStreamRequestFailed, &[&e.to_string()]))?;

    if !response.status().is_success() {
        return Err(fmt_msg(MsgKey::HttpStreamError, &[&response.status().to_string()]));
    }

    if is_bytes {
        // バイナリモード：バイト列として取得
        let bytes = response
            .bytes()
            .map_err(|e| fmt_msg(MsgKey::HttpStreamReadBytesFailed, &[&e.to_string()]))?;

        // 4KBチャンクに分割
        const CHUNK_SIZE: usize = 4096;
        let chunks: Vec<Vec<u8>> = bytes
            .chunks(CHUNK_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();

        let index = Arc::new(RwLock::new(0));

        let stream = Stream {
            next_fn: Box::new(move || {
                let mut idx = index.write();
                if *idx < chunks.len() {
                    let chunk = &chunks[*idx];
                    *idx += 1;
                    // バイト配列をIntegerのVectorに変換
                    let bytes: Vec<Value> = chunk.iter().map(|&b| Value::Integer(b as i64)).collect();
                    Some(Value::Vector(bytes))
                } else {
                    None
                }
            }),
        };

        Ok(Value::Stream(Arc::new(RwLock::new(stream))))
    } else {
        // テキストモード：行ごと
        let body = response
            .text()
            .map_err(|e| fmt_msg(MsgKey::HttpStreamReadBodyFailed, &[&e.to_string()]))?;

        // 行ごとに分割してストリームに変換
        let lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();
        let index = Arc::new(RwLock::new(0));

        let stream = Stream {
            next_fn: Box::new(move || {
                let mut idx = index.write();
                if *idx < lines.len() {
                    let line = lines[*idx].clone();
                    *idx += 1;
                    Some(Value::String(line))
                } else {
                    None
                }
            }),
        };

        Ok(Value::Stream(Arc::new(RwLock::new(stream))))
    }
}
