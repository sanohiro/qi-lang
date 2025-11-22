use super::*;

/// HTTPリクエストの実装（シンプル版：bodyの文字列のみ返す）
pub(super) fn http_request(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: Option<&crate::HashMap<crate::value::MapKey, Value>>,
    timeout_ms: u64,
) -> Result<Value, String> {
    // 詳細版を呼び出す
    let result = http_request_detailed(method, url, body, headers, timeout_ms)?;

    // 詳細版の戻り値を処理
    match result {
        Value::Map(m) => {
            // キーを準備
            let status_key = Value::Keyword(crate::intern::intern_keyword("status"))
                .to_map_key()
                .expect("status keyword should be valid");
            let body_key = Value::Keyword(crate::intern::intern_keyword("body"))
                .to_map_key()
                .expect("body keyword should be valid");
            let error_key = Value::Keyword(crate::intern::intern_keyword("error"))
                .to_map_key()
                .expect("error keyword should be valid");

            // errorキーがある場合（ネットワークエラー等）
            if let Some(Value::Map(err_map)) = m.get(&error_key) {
                let message_key = Value::Keyword(crate::intern::intern_keyword("message"))
                    .to_map_key()
                    .expect("message keyword should be valid");

                if let Some(Value::String(msg)) = err_map.get(&message_key) {
                    return Err(msg.clone());
                }
                return Err(fmt_msg(MsgKey::HttpUnexpectedErrorFormat, &[]));
            }

            // statusキーとbodyキーを取得
            let status = m
                .get(&status_key)
                .and_then(|v| match v {
                    Value::Integer(i) => Some(*i),
                    _ => None,
                })
                .ok_or_else(|| fmt_msg(MsgKey::HttpMissingStatus, &[]))?;

            let body_val = m
                .get(&body_key)
                .ok_or_else(|| fmt_msg(MsgKey::HttpMissingBody, &[]))?;

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
                    Err(fmt_msg(
                        MsgKey::HttpErrorWithBody,
                        &[&status.to_string(), &body_preview],
                    ))
                }
            }
        }
        _ => Err(fmt_msg(MsgKey::HttpUnexpectedResponse, &[])),
    }
}

/// HTTPリクエストの実装（詳細版：Map形式で詳細情報を返す）
pub(super) fn http_request_detailed(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: Option<&crate::HashMap<crate::value::MapKey, Value>>,
    timeout_ms: u64,
) -> Result<Value, String> {
    // デフォルトタイムアウト（30秒）の場合は共有Clientを使用
    // カスタムタイムアウトの場合のみ新しいClientを作成
    let custom_client;
    let client = if timeout_ms == 30000 {
        crate::builtins::lazy_init::http_client::get_client()?
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
        .and_then(|h| {
            h.get(&crate::value::MapKey::String(
                "content-encoding".to_string(),
            ))
        })
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
                // MapKeyから文字列を取得
                let key = match k {
                    crate::value::MapKey::String(s) => s.trim_matches('"'),
                    crate::value::MapKey::Symbol(s) => s.as_ref(),
                    crate::value::MapKey::Keyword(s) => s.as_ref(),
                    crate::value::MapKey::Integer(i) => &i.to_string(),
                };
                let value = val.trim_matches('"');
                request = request.header(key, value);
            }
        }
    }

    // ボディ追加
    if let Some(b) = body {
        match b {
            Value::Bytes(data) => {
                // バイナリデータをそのまま送信
                if should_compress {
                    let compressed = helpers::compress_gzip(data.as_ref())
                        .map_err(|e| fmt_msg(MsgKey::HttpCompressionError, &[&e.to_string()]))?;
                    request = request.header("Content-Encoding", "gzip").body(compressed);
                } else {
                    request = request.body(data.as_ref().to_vec());
                }
            }
            Value::String(s) => {
                if should_compress {
                    // gzip圧縮して送信
                    let compressed = helpers::compress_gzip(s.as_bytes())
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
                        let compressed = helpers::compress_gzip(s.as_bytes()).map_err(|e| {
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
            let headers: crate::HashMap<crate::value::MapKey, Value> = response
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        crate::value::MapKey::String(k.as_str().to_string()),
                        Value::String(v.to_str().unwrap_or("").to_string()),
                    )
                })
                .collect();

            // ボディをバイナリとして取得
            let body_bytes = response.bytes().unwrap_or_default();

            // UTF-8として解釈を試み、成功すれば文字列、失敗すればBytesとして返す
            let body_value = match std::str::from_utf8(&body_bytes) {
                Ok(text) => Value::String(text.to_string()),
                Err(_) => Value::Bytes(std::sync::Arc::from(body_bytes.as_ref())),
            };

            // キーワードキーを生成
            let status_key = Value::Keyword(crate::intern::intern_keyword("status"))
                .to_map_key()
                .expect("status keyword should be valid");
            let headers_key = Value::Keyword(crate::intern::intern_keyword("headers"))
                .to_map_key()
                .expect("headers keyword should be valid");
            let body_key = Value::Keyword(crate::intern::intern_keyword("body"))
                .to_map_key()
                .expect("body keyword should be valid");

            Ok(Value::Map(
                [
                    (status_key, Value::Integer(status)),
                    (headers_key, Value::Map(headers)),
                    (body_key, body_value),
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
            let error_key = Value::Keyword(crate::intern::intern_keyword("error"))
                .to_map_key()
                .expect("error keyword should be valid");
            let type_key = Value::Keyword(crate::intern::intern_keyword("type"))
                .to_map_key()
                .expect("type keyword should be valid");
            let message_key = Value::Keyword(crate::intern::intern_keyword("message"))
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

/// HTTPストリーミングの共通実装（真のストリーミング - メモリに全体を読み込まない）
pub(super) fn http_stream(
    method: &str,
    url: &str,
    body: Option<&Value>,
    is_bytes: bool,
) -> Result<Value, String> {
    // 共有Clientを使用（デフォルトで30秒タイムアウト設定済み）
    let client = crate::builtins::lazy_init::http_client::get_client()?;

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
            Value::Bytes(data) => {
                // バイナリデータをそのまま送信
                request = request.body(data.as_ref().to_vec());
            }
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
