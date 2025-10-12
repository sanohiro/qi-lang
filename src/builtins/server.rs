//! サーバーモジュール
//!
//! HTTPサーバー機能（Flow-Oriented）:
//! - serve: サーバー起動（ルーター対応）
//! - ok/json/not-found/no-content: レスポンスヘルパー
//! - router: ルーティング定義
//! - with-logging/with-cors/with-json-body: ミドルウェア
//! - static-file/static-dir: 静的ファイル配信
//!
//! このモジュールは `http-server` feature でコンパイルされます。

#![cfg(feature = "http-server")]

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use flate2::read::GzDecoder;
use std::io::Read;

/// gzip解凍ヘルパー関数
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// gzip圧縮ヘルパー関数（レスポンス用）
/// latin1形式のString（各char 0-255がu8にマッピング）を圧縮し、
/// 圧縮されたバイナリデータをlatin1形式のStringとして返す
fn compress_gzip_response(body: &str) -> Result<String, std::io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    // latin1形式のStringをバイト列に変換
    let bytes: Vec<u8> = body.chars().map(|c| c as u8).collect();

    // gzip圧縮
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes)?;
    let compressed = encoder.finish()?;

    // 圧縮されたバイト列をlatin1形式のStringに変換
    Ok(compressed.iter().map(|&b| b as char).collect())
}

/// クエリパラメータをパース
/// ?page=1&limit=10 → {"page": "1", "limit": "10"}
/// ?tag=a&tag=b → {"tag": ["a", "b"]}
fn parse_query_params(query_str: &str) -> HashMap<String, Value> {
    let mut params: HashMap<String, Vec<String>> = HashMap::new();

    if query_str.is_empty() {
        return HashMap::new();
    }

    // &で分割してkey=value形式をパース
    for pair in query_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // URLデコード
            let decoded_key = urlencoding::decode(key).unwrap_or(std::borrow::Cow::Borrowed(key));
            let decoded_value = urlencoding::decode(value).unwrap_or(std::borrow::Cow::Borrowed(value));

            params.entry(decoded_key.to_string())
                .or_insert_with(Vec::new)
                .push(decoded_value.to_string());
        } else {
            // 値がない場合（?flag）は空文字列
            let decoded_key = urlencoding::decode(pair).unwrap_or(std::borrow::Cow::Borrowed(pair));
            params.entry(decoded_key.to_string())
                .or_insert_with(Vec::new)
                .push(String::new());
        }
    }

    // 同じキーが複数ある場合は配列、1つの場合は文字列
    params.into_iter()
        .map(|(k, v)| {
            let value = if v.len() == 1 {
                Value::String(v[0].clone())
            } else {
                Value::Vector(v.into_iter().map(Value::String).collect())
            };
            (k, value)
        })
        .collect()
}

/// HTTPリクエストをQi値に変換
async fn request_to_value(req: Request<hyper::body::Incoming>) -> Result<(Value, String), String> {
    let (parts, body) = req.into_parts();

    // ボディを取得（非同期）
    let body_bytes = body
        .collect()
        .await
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToReadBody, &[&e.to_string()]))?
        .to_bytes();

    // Content-Encodingヘッダーをチェックして解凍
    let decompressed_bytes = if let Some(encoding) = parts.headers.get("content-encoding") {
        if let Ok(encoding_str) = encoding.to_str() {
            if encoding_str.to_lowercase() == "gzip" {
                // gzip解凍
                decompress_gzip(&body_bytes)
                    .map_err(|e| fmt_msg(MsgKey::ServerFailedToDecompressGzip, &[&e.to_string()]))?
            } else {
                body_bytes.to_vec()
            }
        } else {
            body_bytes.to_vec()
        }
    } else {
        body_bytes.to_vec()
    };

    let body_str = String::from_utf8_lossy(&decompressed_bytes).to_string();

    // メソッド
    let method = parts.method.as_str().to_lowercase();

    // パス
    let path = parts.uri.path().to_string();

    // クエリパラメータ
    let query = parts.uri.query().unwrap_or("").to_string();
    let query_params = parse_query_params(&query);

    // ヘッダー
    let mut headers = HashMap::new();
    for (name, value) in parts.headers.iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(
                name.as_str().to_string(),
                Value::String(v.to_string()),
            );
        }
    }

    // リクエストマップ
    let mut req_map = HashMap::new();
    req_map.insert("method".to_string(), Value::Keyword(method));
    req_map.insert("path".to_string(), Value::String(path));
    req_map.insert("query".to_string(), Value::String(query));
    req_map.insert("query-params".to_string(), Value::Map(query_params));
    req_map.insert("headers".to_string(), Value::Map(headers));
    req_map.insert("body".to_string(), Value::String(body_str.clone()));

    Ok((Value::Map(req_map), body_str))
}

/// Qi値をHTTPレスポンスに変換
fn value_to_response(value: Value) -> Result<Response<Full<Bytes>>, String> {
    match value {
        Value::Map(m) => {
            // {:status 200, :headers {...}, :body "..."}
            let status = match m.get("status") {
                Some(Value::Integer(s)) => *s as u16,
                _ => 200,
            };

            let body_str = match m.get("body") {
                Some(Value::String(s)) => s.clone(),
                Some(v) => format!("{}", v),
                None => String::new(),
            };

            // バイナリデータはlatin1としてStringに格納されているので、バイトに戻す
            // latin1: 各char (0-255) を u8 にマッピング
            let body_bytes: Vec<u8> = body_str.chars().map(|c| c as u8).collect();

            let mut response = Response::builder().status(status);

            // ヘッダー設定
            if let Some(Value::Map(headers)) = m.get("headers") {
                for (k, v) in headers {
                    if let Value::String(val) = v {
                        response = response.header(k.as_str(), val.as_str());
                    }
                }
            }

            response
                .body(Full::new(Bytes::from(body_bytes)))
                .map_err(|e| fmt_msg(MsgKey::ServerFailedToBuildResponse, &[&e.to_string()]))
        }
        _ => Err(fmt_msg(MsgKey::ServerHandlerMustReturnMap, &[value.type_name()])),
    }
}

/// server/ok - 200 OKレスポンスを作成
pub fn native_server_ok(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/ok", "1"]));
    }

    let body = match &args[0] {
        Value::String(s) => s.clone(),
        v => format!("{}", v),
    };

    let mut resp = HashMap::new();
    resp.insert("status".to_string(), Value::Integer(200));
    resp.insert("body".to_string(), Value::String(body));

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), Value::String("text/plain; charset=utf-8".to_string()));
    resp.insert("headers".to_string(), Value::Map(headers));

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
        Value::Map(m) => match m.get("ok") {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(fmt_msg(MsgKey::JsonStringifyError, &[])),
        },
        _ => return Err(fmt_msg(MsgKey::JsonStringifyError, &[])),
    };

    let status = if args.len() > 1 {
        if let Value::Map(opts) = &args[1] {
            match opts.get("status") {
                Some(Value::Integer(s)) => *s,
                _ => 200,
            }
        } else {
            200
        }
    } else {
        200
    };

    let mut resp = HashMap::new();
    resp.insert("status".to_string(), Value::Integer(status));
    resp.insert("body".to_string(), Value::String(json_str));

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), Value::String("application/json; charset=utf-8".to_string()));
    resp.insert("headers".to_string(), Value::Map(headers));

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

    let mut resp = HashMap::new();
    resp.insert("status".to_string(), Value::Integer(404));
    resp.insert("body".to_string(), Value::String(body));

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), Value::String("text/plain; charset=utf-8".to_string()));
    resp.insert("headers".to_string(), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/no-content - 204 No Contentレスポンスを作成
pub fn native_server_no_content(_args: &[Value]) -> Result<Value, String> {
    let mut resp = HashMap::new();
    resp.insert("status".to_string(), Value::Integer(204));
    resp.insert("body".to_string(), Value::String(String::new()));
    Ok(Value::Map(resp))
}

/// server/router - ルーターを作成
pub fn native_server_router(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/router", "1"]));
    }

    // ルート定義をそのまま返す（後でserveで使用）
    // ルートは [[path {:get handler, :post handler}], ...] の形式
    Ok(args[0].clone())
}

/// server/serve - HTTPサーバーを起動
pub fn native_server_serve(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/serve", "1"]));
    }

    let handler = args[0].clone();

    // オプション引数
    let opts = if args.len() > 1 {
        match &args[1] {
            Value::Map(m) => m.clone(),
            _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["server/serve", "a map"])),
        }
    } else {
        HashMap::new()
    };

    let port = match opts.get("port") {
        Some(Value::Integer(p)) => *p as u16,
        _ => 3000,
    };

    let host = match opts.get("host") {
        Some(Value::String(h)) => h.clone(),
        _ => "127.0.0.1".to_string(),
    };

    let timeout_secs = match opts.get("timeout") {
        Some(Value::Integer(t)) => *t as u64,
        _ => 30,
    };

    // サーバーを別スレッドで起動
    let host_clone = host.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            if let Err(e) = run_server(&host_clone, port, handler, timeout_secs).await {
                eprintln!("Server error: {}", e);
            }
        });
    });

    println!("HTTP server started on http://{}:{} (timeout: {}s)", host, port, timeout_secs);

    Ok(Value::Nil)
}

/// サーバー実行
async fn run_server(host: &str, port: u16, handler: Value, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    // ハンドラーをArcで共有
    let handler = Arc::new(handler);
    let timeout_duration = Duration::from_secs(timeout_secs);

    // グレースフルシャットダウン用のシグナル
    let shutdown = Arc::new(tokio::sync::Notify::new());
    let shutdown_clone = shutdown.clone();

    // シグナルハンドラー（Ctrl+C）
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("\nReceived shutdown signal, gracefully shutting down...");
                shutdown_clone.notify_waiters();
            }
            Err(err) => {
                eprintln!("Error listening for shutdown signal: {}", err);
            }
        }
    });

    loop {
        tokio::select! {
            // 新規接続を受け付ける
            result = listener.accept() => {
                let (stream, _) = result?;
                let io = TokioIo::new(stream);
                let handler = handler.clone();
                let timeout = timeout_duration;

                tokio::task::spawn(async move {
                    let service = service_fn(move |req| {
                        let handler = handler.clone();
                        let timeout = timeout;
                        async move {
                            handle_request(req, handler, timeout).await
                        }
                    });

                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            // シャットダウンシグナルを受信
            _ = shutdown.notified() => {
                println!("Server stopped");
                return Ok(());
            }
        }
    }
}

/// リクエスト処理
async fn handle_request(
    req: Request<hyper::body::Incoming>,
    handler: Arc<Value>,
    timeout: Duration,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // タイムアウト付きで処理
    let result = tokio::time::timeout(timeout, async {
        // リクエストをQi値に変換
        let (req_value, _body) = request_to_value(req).await?;

        // ハンドラーがルーター（Vector）の場合、ルーティング処理
        let resp_value = match handler.as_ref() {
            Value::Vector(routes) => route_request(&req_value, routes),
            Value::Function(_) => {
                // 直接関数を呼び出す
                let eval = Evaluator::new();
                eval.apply_function(handler.as_ref(), &[req_value])
            }
            _ => Err(fmt_msg(MsgKey::ServerHandlerMustBeFunction, &[handler.type_name()])),
        };

        // Qi値をHTTPレスポンスに変換
        match resp_value {
            Ok(v) => value_to_response(v),
            Err(e) => {
                eprintln!("Handler error: {}", e);
                Err(fmt_msg(MsgKey::ServerHandlerError, &[&e]))
            }
        }
    }).await;

    match result {
        Ok(Ok(resp)) => Ok(resp),
        Ok(Err(e)) => {
            eprintln!("Error: {}", e);
            Ok(error_response(500, "Internal Server Error"))
        }
        Err(_) => {
            eprintln!("Request timeout");
            Ok(error_response(408, "Request Timeout"))
        }
    }
}

/// ルーティング処理
fn route_request(req: &Value, routes: &[Value]) -> Result<Value, String> {
    let method = match req {
        Value::Map(m) => match m.get("method") {
            Some(Value::Keyword(k)) => k.clone(),
            _ => return Err(fmt_msg(MsgKey::RequestMustHave, &["request", ":method keyword"])),
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    let path = match req {
        Value::Map(m) => match m.get("path") {
            Some(Value::String(p)) => p.clone(),
            _ => return Err(fmt_msg(MsgKey::RequestMustHave, &["request", ":path string"])),
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    // ルートを探索
    for route in routes {
        if let Value::Vector(route_def) = route {
            if route_def.len() == 2 {
                if let (Value::String(pattern), Value::Map(handlers)) = (&route_def[0], &route_def[1]) {
                    // メソッドに対応するハンドラーを取得
                    if let Some(handler) = handlers.get(&method) {
                        // 静的ファイルハンドラーの場合はプレフィックスマッチング
                        if let Value::Map(m) = handler {
                            if m.contains_key("__static_dir__") {
                                // プレフィックスマッチング（パスがパターンで始まっているか）
                                let pattern_normalized = if pattern == "/" { "/" } else { pattern.trim_end_matches('/') };
                                let path_normalized = path.trim_end_matches('/');

                                if path_normalized == pattern_normalized ||
                                   (pattern_normalized == "/" && !path.is_empty()) ||
                                   path.starts_with(&format!("{}/", pattern_normalized)) {
                                    // 静的ファイルハンドラーを実行
                                    let eval = Evaluator::new();
                                    return apply_middleware(handler, req, &eval);
                                }
                                continue;
                            }
                        }

                        // 通常のパスパラメータ対応のパターンマッチング
                        if let Some(params) = match_route_pattern(pattern, &path) {
                            // パラメータをリクエストに追加
                            let mut req_with_params = match req {
                                Value::Map(m) => m.clone(),
                                _ => HashMap::new(),
                            };
                            req_with_params.insert("params".to_string(), Value::Map(params));

                            // ミドルウェアを適用してハンドラーを実行
                            let eval = Evaluator::new();
                            return apply_middleware(handler, &Value::Map(req_with_params), &eval);
                        }
                    }
                }
            }
        }
    }

    // ルートが見つからない
    native_server_not_found(&[])
}

/// 静的ファイルを配信
fn serve_static_file(dir_path: &str, req: &Value) -> Result<Value, String> {
    let path = match req {
        Value::Map(m) => match m.get("path") {
            Some(Value::String(p)) => p,
            _ => return Err(fmt_msg(MsgKey::RequestMustHave, &["request", ":path string"])),
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    // セキュリティチェック
    if !is_safe_path(path) {
        return Err(fmt_msg(MsgKey::InvalidFilePath, &["serve_static_file"]));
    }

    // ファイルパスを構築
    let file_path = std::path::Path::new(dir_path).join(path.trim_start_matches('/'));

    // index.htmlの自動配信（ディレクトリの場合）
    let file_path = if file_path.is_dir() {
        file_path.join("index.html")
    } else {
        file_path
    };

    // ファイルサイズチェック
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                return format!("File not found: {}", path);
            }
            format!("Failed to read file metadata: {}", e)
        })?;

    let file_size = metadata.len();
    if file_size > MAX_STATIC_FILE_SIZE {
        return Err(fmt_msg(
            MsgKey::ServerFileTooLarge,
            &[
                &file_size.to_string(),
                &MAX_STATIC_FILE_SIZE.to_string(),
                &(MAX_STATIC_FILE_SIZE / 1024 / 1024).to_string(),
                path
            ]
        ));
    }

    // ファイル読み込み
    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let content_type = get_content_type(file_path.to_str().unwrap_or(""));

            // バイナリデータを latin1 として String に変換（データロスなし）
            // 各バイト (0-255) を char にマッピング
            let body = bytes.iter().map(|&b| b as char).collect::<String>();

            let mut resp = HashMap::new();
            resp.insert("status".to_string(), Value::Integer(200));
            resp.insert("body".to_string(), Value::String(body));

            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), Value::String(content_type.to_string()));
            resp.insert("headers".to_string(), Value::Map(headers));

            Ok(Value::Map(resp))
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                native_server_not_found(&[Value::String(format!("File not found: {}", path))])
            } else {
                Err(fmt_msg(MsgKey::ServerFailedToReadFile, &[&e.to_string()]))
            }
        }
    }
}

/// ミドルウェアを適用してハンドラーを実行
fn apply_middleware(handler: &Value, req: &Value, eval: &Evaluator) -> Result<Value, String> {
    // 静的ファイルハンドラーかチェック
    if let Value::Map(m) = handler {
        if let Some(Value::String(dir_path)) = m.get("__static_dir__") {
            // 静的ファイル配信
            return serve_static_file(dir_path, req);
        }

        if let Some(Value::String(middleware_type)) = m.get("__middleware__") {
            // ミドルウェアの場合、内部のハンドラーを取得
            if let Some(inner_handler) = m.get("__handler__") {
                // Basic Auth検証
                if middleware_type == "basic-auth" {
                    if let Value::Map(req_map) = req {
                        let authorized = if let Some(Value::Map(users)) = m.get("__users__") {
                            // Authorizationヘッダーを取得
                            let auth_header = req_map
                                .get("headers")
                                .and_then(|h| match h {
                                    Value::Map(headers) => headers.get("authorization"),
                                    _ => None,
                                })
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                });

                            if let Some(auth) = auth_header {
                                if auth.starts_with("Basic ") {
                                    use base64::{Engine as _, engine::general_purpose};
                                    let encoded = auth.strip_prefix("Basic ").unwrap();  // "Basic " を除く
                                    if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(encoded) {
                                        if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                                            if let Some((user, pass)) = decoded.split_once(':') {
                                                // ユーザー名とパスワードを検証
                                                users.get(user)
                                                    .and_then(|v| match v {
                                                        Value::String(expected_pass) => Some(pass == expected_pass),
                                                        _ => None,
                                                    })
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if !authorized {
                            // 401 Unauthorized を返す
                            let mut resp = HashMap::new();
                            resp.insert("status".to_string(), Value::Integer(401));
                            resp.insert("body".to_string(), Value::String("Unauthorized".to_string()));
                            let mut headers = HashMap::new();
                            headers.insert("WWW-Authenticate".to_string(), Value::String("Basic realm=\"Restricted\"".to_string()));
                            resp.insert("headers".to_string(), Value::Map(headers));
                            return Ok(Value::Map(resp));
                        }
                    }
                }

                // リクエストを前処理（json-body, bearer）
                let processed_req = match middleware_type.as_str() {
                    "json-body" => {
                        // リクエストボディをJSONパース
                        if let Value::Map(req_map) = req {
                            if let Some(Value::String(body)) = req_map.get("body") {
                                if !body.is_empty() {
                                    match crate::builtins::json::native_parse(&[Value::String(body.clone())]) {
                                        Ok(Value::Map(result)) => {
                                            if let Some(json_value) = result.get("ok") {
                                                let mut new_req = req_map.clone();
                                                new_req.insert("json".to_string(), json_value.clone());
                                                Value::Map(new_req)
                                            } else {
                                                req.clone()
                                            }
                                        }
                                        _ => req.clone(),
                                    }
                                } else {
                                    req.clone()
                                }
                            } else {
                                req.clone()
                            }
                        } else {
                            req.clone()
                        }
                    }
                    "bearer" => {
                        // AuthorizationヘッダーからBearerトークンを抽出
                        if let Value::Map(mut req_map) = req.clone() {
                            let token = req_map
                                .get("headers")
                                .and_then(|h| match h {
                                    Value::Map(headers) => headers.get("authorization"),
                                    _ => None,
                                })
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .and_then(|auth| {
                                    if auth.starts_with("Bearer ") {
                                        Some(auth.strip_prefix("Bearer ").unwrap().to_string())  // "Bearer " を除く
                                    } else {
                                        None
                                    }
                                });

                            if let Some(t) = token {
                                req_map.insert("bearer-token".to_string(), Value::String(t));
                            }
                            Value::Map(req_map)
                        } else {
                            req.clone()
                        }
                    }
                    _ => req.clone(),
                };

                // ロギング（リクエスト）
                if middleware_type == "logging" {
                    if let Value::Map(req_map) = &processed_req {
                        let method = req_map.get("method")
                            .and_then(|v| match v {
                                Value::Keyword(k) => Some(k.to_uppercase()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let path = req_map.get("path")
                            .and_then(|v| match v {
                                Value::String(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        println!("[HTTP] {} {}", method, path);
                    }
                }

                // 内部ハンドラーを再帰的に実行（ネストしたミドルウェア対応）
                let response = apply_middleware(inner_handler, &processed_req, eval)?;

                // レスポンスを後処理（cors, compression, logging）
                let processed_resp = match middleware_type.as_str() {
                    "cors" => {
                        // CORSヘッダーを追加
                        if let Value::Map(mut resp_map) = response.clone() {
                            let origins = m.get("__origins__")
                                .and_then(|v| match v {
                                    Value::Vector(v) => Some(v.clone()),
                                    _ => None,
                                })
                                .unwrap_or_else(|| vec![Value::String("*".to_string())]);

                            let origin = origins.first()
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .unwrap_or_else(|| "*".to_string());

                            let mut headers = match resp_map.get("headers") {
                                Some(Value::Map(h)) => h.clone(),
                                _ => HashMap::new(),
                            };

                            headers.insert("Access-Control-Allow-Origin".to_string(), Value::String(origin));
                            headers.insert("Access-Control-Allow-Methods".to_string(),
                                Value::String("GET, POST, PUT, DELETE, OPTIONS".to_string()));
                            headers.insert("Access-Control-Allow-Headers".to_string(),
                                Value::String("Content-Type, Authorization".to_string()));

                            resp_map.insert("headers".to_string(), Value::Map(headers));
                            Value::Map(resp_map)
                        } else {
                            response
                        }
                    }
                    "compression" => {
                        // レスポンスボディを圧縮
                        if let Value::Map(mut resp_map) = response.clone() {
                            let min_size = m.get("__min_size__")
                                .and_then(|v| match v {
                                    Value::Integer(s) => Some(*s as usize),
                                    _ => None,
                                })
                                .unwrap_or(1024);

                            // ボディを取得
                            if let Some(Value::String(body)) = resp_map.get("body") {
                                // ボディサイズチェック（latin1形式なので、char数 = byte数）
                                if body.len() >= min_size {
                                    // gzip圧縮
                                    match compress_gzip_response(body) {
                                        Ok(compressed_body) => {
                                            // 圧縮されたボディを設定
                                            resp_map.insert("body".to_string(), Value::String(compressed_body));

                                            // Content-Encodingヘッダーを追加
                                            let mut headers = match resp_map.get("headers") {
                                                Some(Value::Map(h)) => h.clone(),
                                                _ => HashMap::new(),
                                            };
                                            headers.insert("Content-Encoding".to_string(), Value::String("gzip".to_string()));
                                            resp_map.insert("headers".to_string(), Value::Map(headers));
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to compress response: {}", e);
                                            // 圧縮失敗時は元のレスポンスを返す
                                        }
                                    }
                                }
                            }

                            Value::Map(resp_map)
                        } else {
                            response
                        }
                    }
                    "logging" => {
                        // レスポンスステータスをログ出力
                        if let Value::Map(resp_map) = &response {
                            let status = resp_map.get("status")
                                .and_then(|v| match v {
                                    Value::Integer(i) => Some(*i),
                                    _ => None,
                                })
                                .unwrap_or(200);
                            println!("[HTTP] -> {}", status);
                        }
                        response
                    }
                    "no-cache" => {
                        // キャッシュ無効化ヘッダーを追加
                        if let Value::Map(mut resp_map) = response.clone() {
                            let mut headers = match resp_map.get("headers") {
                                Some(Value::Map(h)) => h.clone(),
                                _ => HashMap::new(),
                            };

                            headers.insert("Cache-Control".to_string(),
                                Value::String("no-store, no-cache, must-revalidate, private".to_string()));
                            headers.insert("Pragma".to_string(),
                                Value::String("no-cache".to_string()));
                            headers.insert("Expires".to_string(),
                                Value::String("0".to_string()));

                            resp_map.insert("headers".to_string(), Value::Map(headers));
                            Value::Map(resp_map)
                        } else {
                            response
                        }
                    }
                    "cache-control" => {
                        // カスタムCache-Controlヘッダーを追加
                        if let Value::Map(mut resp_map) = response.clone() {
                            if let Some(Value::Map(opts)) = m.get("__cache_opts__") {
                                let mut cache_parts = Vec::new();

                                // max-age
                                if let Some(Value::Integer(age)) = opts.get("max-age") {
                                    cache_parts.push(format!("max-age={}", age));
                                }

                                // public/private
                                if let Some(Value::Bool(true)) = opts.get("public") {
                                    cache_parts.push("public".to_string());
                                } else if let Some(Value::Bool(true)) = opts.get("private") {
                                    cache_parts.push("private".to_string());
                                }

                                // no-store
                                if let Some(Value::Bool(true)) = opts.get("no-store") {
                                    cache_parts.push("no-store".to_string());
                                }

                                // must-revalidate
                                if let Some(Value::Bool(true)) = opts.get("must-revalidate") {
                                    cache_parts.push("must-revalidate".to_string());
                                }

                                // immutable
                                if let Some(Value::Bool(true)) = opts.get("immutable") {
                                    cache_parts.push("immutable".to_string());
                                }

                                if !cache_parts.is_empty() {
                                    let cache_control = cache_parts.join(", ");
                                    let mut headers = match resp_map.get("headers") {
                                        Some(Value::Map(h)) => h.clone(),
                                        _ => HashMap::new(),
                                    };
                                    headers.insert("Cache-Control".to_string(), Value::String(cache_control));
                                    resp_map.insert("headers".to_string(), Value::Map(headers));
                                }
                            }

                            Value::Map(resp_map)
                        } else {
                            response
                        }
                    }
                    _ => response,
                };

                return Ok(processed_resp);
            }
        }
    }

    // ミドルウェアでない場合、直接ハンドラーを実行
    eval.apply_function(handler, &[req.clone()])
}

/// パスパターンマッチング - /users/:id のような形式をサポート
/// 戻り値: マッチした場合はパラメータマップ、マッチしない場合はNone
fn match_route_pattern(pattern: &str, path: &str) -> Option<HashMap<String, Value>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // パート数が異なる場合はマッチしない
    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    // 各パートを比較
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if pattern_part.starts_with(':') {
            // パラメータ部分 - パラメータ名を抽出
            let param_name = pattern_part.strip_prefix(':').unwrap();  // ':' を除く
            params.insert(param_name.to_string(), Value::String(path_part.to_string()));
        } else if pattern_part != path_part {
            // 固定部分が一致しない
            return None;
        }
    }

    Some(params)
}

/// エラーレスポンス生成
fn error_response(status: u16, message: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Full::new(Bytes::from(message.to_string())))
        .unwrap()
}

// ========================================
// ミドルウェアシステム
// ========================================

/// server/with-logging - ロギングミドルウェア
/// リクエスト/レスポンスをログ出力
pub fn native_server_with_logging(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-logging"]));
    }

    let handler = args[0].clone();

    // ロギングミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("logging".to_string()));
    metadata.insert("__handler__".to_string(), handler);

    Ok(Value::Map(metadata))
}

/// server/with-cors - CORSミドルウェア
/// CORSヘッダーを追加
pub fn native_server_with_cors(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-cors"]));
    }

    let handler = args[0].clone();

    // オプション引数（CORS設定）
    let origins = if args.len() > 1 {
        match &args[1] {
            Value::Map(m) => match m.get("origins") {
                Some(Value::Vector(v)) => v.iter()
                    .filter_map(|val| match val {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => vec!["*".to_string()],
            },
            _ => vec!["*".to_string()],
        }
    } else {
        vec!["*".to_string()]
    };

    // CORSミドルウェアマーカーとして、マップにメタデータを埋め込む
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("cors".to_string()));
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert("__origins__".to_string(), Value::Vector(
        origins.iter().map(|s| Value::String(s.clone())).collect()
    ));

    Ok(Value::Map(metadata))
}

/// server/with-json-body - JSONボディ自動パースミドルウェア
/// リクエストボディを自動的にJSONパース
pub fn native_server_with_json_body(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-json-body"]));
    }

    let handler = args[0].clone();

    // JSONボディパースミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("json-body".to_string()));
    metadata.insert("__handler__".to_string(), handler);

    Ok(Value::Map(metadata))
}

/// server/with-compression - レスポンス圧縮ミドルウェア
/// レスポンスボディをgzip圧縮
pub fn native_server_with_compression(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-compression"]));
    }

    let handler = args[0].clone();

    // オプション引数（圧縮設定）
    let min_size = if args.len() > 1 {
        match &args[1] {
            Value::Map(m) => match m.get("min-size") {
                Some(Value::Integer(s)) => *s as usize,
                _ => 1024, // デフォルト: 1KB以上で圧縮
            },
            _ => 1024,
        }
    } else {
        1024
    };

    // 圧縮ミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("compression".to_string()));
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert("__min_size__".to_string(), Value::Integer(min_size as i64));

    Ok(Value::Map(metadata))
}

// ========================================
// 静的ファイル配信
// ========================================

/// 拡張子からContent-Typeを判定
fn get_content_type(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",

        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",

        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",

        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",

        _ => "application/octet-stream",
    }
}

/// パストラバーサル攻撃をチェック
fn is_safe_path(path: &str) -> bool {
    // .. を含むパスは拒否
    !path.contains("..") && !path.contains("//")
}

/// 静的ファイルの最大サイズ（10MB）
/// この制限は、ファイル全体をメモリに読み込むことによるOOM防止のため
const MAX_STATIC_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// server/static-file - 単一ファイルを配信するレスポンスを生成
pub fn native_server_static_file(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-file"]));
    }

    let file_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["server/static-file", "file path"])),
    };

    // セキュリティチェック
    if !is_safe_path(file_path) {
        return Err(fmt_msg(MsgKey::InvalidFilePath, &["server/static-file"]));
    }

    // ファイルサイズチェック
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| fmt_msg(MsgKey::ServerStaticFileMetadataFailed, &[&e.to_string()]))?;

    let file_size = metadata.len();
    if file_size > MAX_STATIC_FILE_SIZE {
        return Err(fmt_msg(
            MsgKey::ServerStaticFileTooLarge,
            &[
                &file_size.to_string(),
                &MAX_STATIC_FILE_SIZE.to_string(),
                &(MAX_STATIC_FILE_SIZE / 1024 / 1024).to_string()
            ]
        ));
    }

    // ファイル読み込み
    match std::fs::read(file_path) {
        Ok(bytes) => {
            let content_type = get_content_type(file_path);

            // バイナリデータを latin1 として String に変換（データロスなし）
            // 各バイト (0-255) を char にマッピング
            let body = bytes.iter().map(|&b| b as char).collect::<String>();

            let mut resp = HashMap::new();
            resp.insert("status".to_string(), Value::Integer(200));
            resp.insert("body".to_string(), Value::String(body));

            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), Value::String(content_type.to_string()));
            resp.insert("headers".to_string(), Value::Map(headers));

            Ok(Value::Map(resp))
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                native_server_not_found(&[Value::String(format!("File not found: {}", file_path))])
            } else {
                Err(fmt_msg(MsgKey::ServerStaticFileFailedToRead, &[&e.to_string()]))
            }
        }
    }
}

/// server/static-dir - ディレクトリから静的ファイルを配信するハンドラーを生成
pub fn native_server_static_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-dir"]));
    }

    let dir_path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["server/static-dir", "directory path"])),
    };

    // セキュリティチェック
    if !is_safe_path(&dir_path) {
        return Err(fmt_msg(MsgKey::InvalidFilePath, &["server/static-dir"]));
    }

    // ディレクトリの存在チェック
    if !std::path::Path::new(&dir_path).is_dir() {
        return Err(fmt_msg(MsgKey::ServerStaticDirNotDirectory, &[&dir_path]));
    }

    // 静的ファイルハンドラーマーカー（ミドルウェアと同じパターン）
    let mut metadata = HashMap::new();
    metadata.insert("__static_dir__".to_string(), Value::String(dir_path));

    Ok(Value::Map(metadata))
}

// ========================================
// 認証ミドルウェア
// ========================================

/// server/with-basic-auth - Basic認証ミドルウェア
/// リクエストのAuthorizationヘッダーをチェックし、認証に失敗したら401を返す
pub fn native_server_with_basic_auth(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-basic-auth"]));
    }

    let handler = args[0].clone();

    // ユーザー設定（オプション引数）
    let users = if args.len() > 1 {
        match &args[1] {
            Value::Map(m) => match m.get("users") {
                Some(Value::Map(u)) => u.clone(),
                _ => HashMap::new(),
            },
            _ => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    // Basic Authミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("basic-auth".to_string()));
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert("__users__".to_string(), Value::Map(users));

    Ok(Value::Map(metadata))
}

/// server/with-bearer - Bearer Token抽出ミドルウェア
/// AuthorizationヘッダーからBearerトークンを抽出してreq["bearer-token"]に格納
pub fn native_server_with_bearer(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-bearer"]));
    }

    let handler = args[0].clone();

    // Bearerミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("bearer".to_string()));
    metadata.insert("__handler__".to_string(), handler);

    Ok(Value::Map(metadata))
}

// ========================================
// キャッシュ制御ミドルウェア
// ========================================

/// server/with-no-cache - キャッシュ無効化ミドルウェア
/// レスポンスにキャッシュを無効化するヘッダーを追加
pub fn native_server_with_no_cache(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-no-cache"]));
    }

    let handler = args[0].clone();

    // no-cacheミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("no-cache".to_string()));
    metadata.insert("__handler__".to_string(), handler);

    Ok(Value::Map(metadata))
}

/// server/with-cache-control - カスタムキャッシュ制御ミドルウェア
/// レスポンスにCache-Controlヘッダーを追加
/// オプション: {"max-age" 3600 "public" true "private" false "no-store" false}
pub fn native_server_with_cache_control(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-cache-control"]));
    }

    let handler = args[0].clone();

    // オプション引数（キャッシュ設定）
    let opts = if args.len() > 1 {
        match &args[1] {
            Value::Map(m) => m.clone(),
            _ => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    // cache-controlミドルウェアマーカー
    let mut metadata = HashMap::new();
    metadata.insert("__middleware__".to_string(), Value::String("cache-control".to_string()));
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert("__cache_opts__".to_string(), Value::Map(opts));

    Ok(Value::Map(metadata))
}
