//! HTTPモジュール
//!
//! HTTP通信関数を提供:
//! - get/post/put/delete/patch/head/options: 各HTTPメソッド
//! - request: 詳細なリクエスト設定
//! - get-async/post-async: 非同期版

use crate::eval::Evaluator;
use crate::value::Value;
use crossbeam_channel::bounded;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// HTTP GETリクエスト
pub fn native_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/get: 1個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/get: URLは文字列である必要があります".to_string()),
    };

    http_request("GET", url, None, None, 30000)
}

/// HTTP POSTリクエスト
pub fn native_post(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("http/post: 2個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/post: URLは文字列である必要があります".to_string()),
    };

    http_request("POST", url, Some(&args[1]), None, 30000)
}

/// HTTP PUTリクエスト
pub fn native_put(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("http/put: 2個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/put: URLは文字列である必要があります".to_string()),
    };

    http_request("PUT", url, Some(&args[1]), None, 30000)
}

/// HTTP DELETEリクエスト
pub fn native_delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/delete: 1個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/delete: URLは文字列である必要があります".to_string()),
    };

    http_request("DELETE", url, None, None, 30000)
}

/// HTTP PATCHリクエスト
pub fn native_patch(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("http/patch: 2個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/patch: URLは文字列である必要があります".to_string()),
    };

    http_request("PATCH", url, Some(&args[1]), None, 30000)
}

/// HTTP HEADリクエスト
pub fn native_head(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/head: 1個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/head: URLは文字列である必要があります".to_string()),
    };

    http_request("HEAD", url, None, None, 30000)
}

/// HTTP OPTIONSリクエスト
pub fn native_options(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/options: 1個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err("http/options: URLは文字列である必要があります".to_string()),
    };

    http_request("OPTIONS", url, None, None, 30000)
}

/// 詳細なHTTPリクエスト
pub fn native_request(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/request: 1個の引数が必要です".to_string());
    }

    let opts = match &args[0] {
        Value::Map(m) => m,
        _ => return Err("http/request: 引数はマップである必要があります".to_string()),
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
        .ok_or("http/request: :url は必須です")?;

    let body = opts.get("body");

    let headers = opts.get("headers").and_then(|v| match v {
        Value::Map(m) => Some(m),
        _ => None,
    });

    let timeout = opts
        .get("timeout")
        .and_then(|v| match v {
            Value::Integer(i) if *i > 0 => Some(*i as u64),
            _ => None,
        })
        .unwrap_or(30000);

    http_request(method, url, body, headers, timeout)
}

/// HTTP GETリクエスト (非同期)
pub fn native_get_async(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Err("http/get-async: 1個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("http/get-async: URLは文字列である必要があります".to_string()),
    };

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: sender.clone(),
        receiver,
    });

    std::thread::spawn(move || {
        let result = native_get(&[Value::String(url)]);
        let _ = sender.send(result.unwrap_or_else(|e| Value::String(e)));
    });

    Ok(Value::Channel(result_channel))
}

/// HTTP POSTリクエスト (非同期)
pub fn native_post_async(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("http/post-async: 2個の引数が必要です".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("http/post-async: URLは文字列である必要があります".to_string()),
    };

    let body = args[1].clone();

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: sender.clone(),
        receiver,
    });

    std::thread::spawn(move || {
        let result = native_post(&[Value::String(url), body]);
        let _ = sender.send(result.unwrap_or_else(|e| Value::String(e)));
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
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| format!("HTTPクライアントエラー: {}", e))?;

    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        "HEAD" => client.head(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        _ => return Err(format!("未サポートのHTTPメソッド: {}", method)),
    };

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
                request = request.body(s.clone());
            }
            _ => {
                // JSON自動変換
                let json_str = crate::builtins::json::native_stringify(&[b.clone()])?;
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
