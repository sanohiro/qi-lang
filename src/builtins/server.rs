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

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use crate::HashMap;
use flate2::read::GzDecoder;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File as TokioFile;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

/// gzip解凍ヘルパー関数
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// gzip圧縮ヘルパー関数（レスポンス用）
/// UTF-8文字列を圧縮し、圧縮されたバイナリデータをバイト列として表現したStringとして返す
fn compress_gzip_response(body: &str) -> Result<String, std::io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    // UTF-8文字列をバイト列に変換
    let bytes = body.as_bytes();

    // gzip圧縮
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes)?;
    let compressed = encoder.finish()?;

    // 圧縮されたバイト列をバイト列表現のStringに変換
    Ok(compressed.iter().map(|&b| b as char).collect())
}

// ========================================
// ミドルウェアヘルパー関数
// ========================================

/// JSONボディをパースしてリクエストに追加
fn apply_json_body_middleware(req: &Value) -> Value {
    let Value::Map(req_map) = req else {
        return req.clone();
    };

    let body_key = Value::Keyword("body".to_string()).to_map_key().unwrap();
    let Some(Value::String(body)) = req_map.get(&body_key) else {
        return req.clone();
    };

    if body.is_empty() {
        return req.clone();
    }

    let Ok(Value::Map(result)) =
        crate::builtins::json::native_parse(&[Value::String(body.clone())])
    else {
        return req.clone();
    };

    let ok_key = Value::Keyword("ok".to_string()).to_map_key().unwrap();
    let Some(json_value) = result.get(&ok_key) else {
        return req.clone();
    };

    // JSON解析成功 → 変更が必要なのでclone
    let mut new_req = req_map.clone();
    let json_key = Value::Keyword("json".to_string()).to_map_key().unwrap();
    new_req.insert(json_key, json_value.clone());
    Value::Map(new_req)
}

/// Bearerトークンを抽出してリクエストに追加
fn apply_bearer_middleware(req: &Value) -> Value {
    let Value::Map(req_map) = req else {
        return req.clone();
    };

    let headers_key = Value::Keyword("headers".to_string()).to_map_key().unwrap();
    let auth_header_key = "authorization"; // HTTPヘッダーは文字列キーのまま

    let token = req_map
        .get(&headers_key)
        .and_then(|h| match h {
            Value::Map(headers) => headers.get(auth_header_key),
            _ => None,
        })
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .and_then(|auth| auth.strip_prefix("Bearer "));

    let Some(token) = token else {
        return req.clone(); // トークンなし → 変更不要
    };

    // トークンあり → 変更が必要なのでclone
    let mut new_req = req_map.clone();
    let bearer_key = Value::Keyword("bearer-token".to_string())
        .to_map_key()
        .unwrap();
    new_req.insert(bearer_key, Value::String(token.to_string()));
    Value::Map(new_req)
}

/// リクエストをロギング
fn apply_logging_middleware(req: &Value) {
    if let Value::Map(req_map) = req {
        let method_key = Value::Keyword("method".to_string()).to_map_key().unwrap();
        let path_key = Value::Keyword("path".to_string()).to_map_key().unwrap();

        let method = req_map
            .get(&method_key)
            .and_then(|v| match v {
                Value::Keyword(k) => Some(k.to_uppercase()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());
        let path = req_map
            .get(&path_key)
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());
        println!("[HTTP] {} {}", method, path);
    }
}

/// CORSヘッダーを追加
fn apply_cors_middleware(resp: &Value, origins: &im::Vector<Value>) -> Value {
    let Value::Map(resp_map) = resp else {
        return resp.clone();
    };

    let headers_key = Value::Keyword("headers".to_string()).to_map_key().unwrap();

    let origin = origins
        .get(0)
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "*".to_string());

    let mut headers = match resp_map.get(&headers_key) {
        Some(Value::Map(h)) => h.clone(),
        _ => crate::new_hashmap(),
    };

    headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        Value::String(origin),
    );
    headers.insert(
        "Access-Control-Allow-Methods".to_string(),
        Value::String("GET, POST, PUT, DELETE, OPTIONS".to_string()),
    );
    headers.insert(
        "Access-Control-Allow-Headers".to_string(),
        Value::String("Content-Type, Authorization".to_string()),
    );

    // CORSヘッダー追加 → 変更が必要なのでclone
    let mut new_resp = resp_map.clone();
    new_resp.insert(headers_key, Value::Map(headers));
    Value::Map(new_resp)
}

/// レスポンスボディを圧縮
fn apply_compression_middleware(resp: &Value, min_size: usize) -> Value {
    let Value::Map(resp_map) = resp else {
        return resp.clone();
    };

    let body_key = Value::Keyword("body".to_string()).to_map_key().unwrap();
    let headers_key = Value::Keyword("headers".to_string()).to_map_key().unwrap();

    let Some(Value::String(body)) = resp_map.get(&body_key) else {
        return resp.clone();
    };

    if body.len() < min_size {
        return resp.clone(); // 圧縮不要 → 変更不要
    }

    let Ok(compressed) = compress_gzip_response(body) else {
        return resp.clone(); // 圧縮失敗 → 変更不要
    };

    // 圧縮成功 → 変更が必要なのでclone
    let mut new_resp = resp_map.clone();
    let mut headers = match resp_map.get(&headers_key) {
        Some(Value::Map(h)) => h.clone(),
        _ => crate::new_hashmap(),
    };
    headers.insert(
        "Content-Encoding".to_string(),
        Value::String("gzip".to_string()),
    );
    new_resp.insert(headers_key.clone(), Value::Map(headers));
    new_resp.insert(body_key, Value::String(compressed));
    Value::Map(new_resp)
}

/// クエリパラメータをパース
/// ?page=1&limit=10 → {"page": "1", "limit": "10"}
/// ?tag=a&tag=b → {"tag": ["a", "b"]}
fn parse_query_params(query_str: &str) -> HashMap<String, Value> {
    let mut params: HashMap<String, Vec<String>> = crate::new_hashmap();

    if query_str.is_empty() {
        return crate::new_hashmap();
    }

    // &で分割してkey=value形式をパース
    for pair in query_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // URLデコード
            let decoded_key = urlencoding::decode(key).unwrap_or(std::borrow::Cow::Borrowed(key));
            let decoded_value =
                urlencoding::decode(value).unwrap_or(std::borrow::Cow::Borrowed(value));

            params
                .entry(decoded_key.to_string())
                .or_default()
                .push(decoded_value.to_string());
        } else {
            // 値がない場合（?flag）は空文字列
            let decoded_key = urlencoding::decode(pair).unwrap_or(std::borrow::Cow::Borrowed(pair));
            params
                .entry(decoded_key.to_string())
                .or_default()
                .push(String::new());
        }
    }

    // 同じキーが複数ある場合は配列、1つの場合は文字列
    params
        .into_iter()
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

    // ヘッダー（HTTPヘッダーは大文字小文字を区別しないため、小文字に正規化）
    let mut headers = crate::new_hashmap();
    for (name, value) in parts.headers.iter() {
        if let Ok(v) = value.to_str() {
            let key = Value::String(name.as_str().to_lowercase()).to_map_key()?;
            headers.insert(key, Value::String(v.to_string()));
        }
    }

    // リクエストマップ
    let mut req_map = crate::new_hashmap();
    req_map.insert(
        Value::Keyword("method".to_string()).to_map_key()?,
        Value::Keyword(method),
    );
    req_map.insert(
        Value::Keyword("path".to_string()).to_map_key()?,
        Value::String(path),
    );
    req_map.insert(
        Value::Keyword("query".to_string()).to_map_key()?,
        Value::String(query),
    );
    req_map.insert(
        Value::Keyword("query-params".to_string()).to_map_key()?,
        Value::Map(query_params),
    );
    req_map.insert(
        Value::Keyword("headers".to_string()).to_map_key()?,
        Value::Map(headers),
    );
    req_map.insert(
        Value::Keyword("body".to_string()).to_map_key()?,
        Value::String(body_str.clone()),
    );

    Ok((Value::Map(req_map), body_str))
}

/// ファイルをストリーミングでレスポンスボディに変換
async fn create_file_stream_body(file_path: &str) -> Result<BoxBody<Bytes, Infallible>, String> {
    // ファイルを非同期で開く
    let file = TokioFile::open(file_path)
        .await
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToReadFile, &[&e.to_string()]))?;

    // ReaderStreamでチャンク単位に読み込み（デフォルト: 8KB chunks）
    let reader_stream = ReaderStream::new(file);

    // StreamをResult<Frame<Bytes>, Infallible>に変換
    let stream = reader_stream.map(|result| match result {
        Ok(bytes) => {
            // Bytes を Frame に変換し、Result でラップ
            Ok::<_, Infallible>(Frame::data(bytes))
        }
        Err(e) => {
            eprintln!("Stream read error: {}", e);
            // エラー時は空フレームを返す
            Ok::<_, Infallible>(Frame::data(Bytes::new()))
        }
    });

    // StreamBodyでBodyを作成
    let body = StreamBody::new(stream);

    // BoxBodyにラップ
    Ok(body.boxed())
}

/// Qi値をHTTPレスポンスに変換
async fn value_to_response(value: Value) -> Result<Response<BoxBody<Bytes, Infallible>>, String> {
    match value {
        Value::Map(m) => {
            // {:status 200, :headers {...}, :body "..." or :body-file "/path"}
            let status_key = Value::Keyword("status".to_string()).to_map_key().unwrap();
            let headers_key = Value::Keyword("headers".to_string()).to_map_key().unwrap();
            let body_key = Value::Keyword("body".to_string()).to_map_key().unwrap();
            let body_file_key = Value::Keyword("body-file".to_string())
                .to_map_key()
                .unwrap();

            let status = match m.get(&status_key) {
                Some(Value::Integer(s)) => *s as u16,
                _ => 200,
            };

            let mut response = Response::builder().status(status);

            // ヘッダー設定
            if let Some(Value::Map(headers)) = m.get(&headers_key) {
                for (k, v) in headers {
                    if let Value::String(val) = v {
                        response = response.header(k.as_str(), val.as_str());
                    }
                }
            }

            // ボディの生成: :body-file が優先
            let body: BoxBody<Bytes, Infallible> =
                if let Some(Value::String(file_path)) = m.get(&body_file_key) {
                    // ファイルストリーミング
                    create_file_stream_body(file_path).await?
                } else {
                    // 従来の :body 処理
                    let body_str = match m.get(&body_key) {
                        Some(Value::String(s)) => s.clone(),
                        Some(v) => format!("{}", v),
                        None => String::new(),
                    };

                    // UTF-8文字列をバイト列に変換
                    // バイナリデータの場合は、バイト列表現のString（各char = byte）からバイトに戻す
                    let body_bytes: Vec<u8> = if body_str.chars().all(|c| c as u32 <= 255) {
                        // バイナリデータ表現の場合
                        body_str.chars().map(|c| c as u8).collect()
                    } else {
                        // 通常のUTF-8文字列の場合
                        body_str.as_bytes().to_vec()
                    };

                    Full::new(Bytes::from(body_bytes)).boxed()
                };

            response
                .body(body)
                .map_err(|e| fmt_msg(MsgKey::ServerFailedToBuildResponse, &[&e.to_string()]))
        }
        _ => Err(fmt_msg(
            MsgKey::ServerHandlerMustReturnMap,
            &[value.type_name()],
        )),
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

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(200),
    );
    resp.insert(
        Value::Keyword("body".to_string()).to_map_key().unwrap(),
        Value::String(body),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

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
        Value::String(s) => s,
        Value::Map(m) if m.contains_key(":error") => {
            return Err(fmt_msg(MsgKey::JsonStringifyError, &[]));
        }
        _ => return Err(fmt_msg(MsgKey::JsonStringifyError, &[])),
    };

    let status = if args.len() > 1 {
        if let Value::Map(opts) = &args[1] {
            let status_key = Value::Keyword("status".to_string()).to_map_key().unwrap();
            match opts.get(&status_key) {
                Some(Value::Integer(s)) => *s,
                _ => 200,
            }
        } else {
            200
        }
    } else {
        200
    };

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(status),
    );
    resp.insert(
        Value::Keyword("body".to_string()).to_map_key().unwrap(),
        Value::String(json_str),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("application/json; charset=utf-8".to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

    Ok(Value::Map(resp))
}

/// server/response - 汎用HTTPレスポンスを作成
/// (server/response status body)
/// status: HTTPステータスコード (Integer)
/// body: レスポンスボディ (String)
pub fn native_server_response(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/response", "2"]));
    }

    let status = match &args[0] {
        Value::Integer(n) => *n,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["server/response", "an integer"],
            ))
        }
    };

    let body = match &args[1] {
        Value::String(s) => s.clone(),
        v => format!("{}", v),
    };

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(status),
    );
    resp.insert(
        Value::Keyword("body".to_string()).to_map_key().unwrap(),
        Value::String(body),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

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

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(404),
    );
    resp.insert(
        Value::Keyword("body".to_string()).to_map_key().unwrap(),
        Value::String(body),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String("text/plain; charset=utf-8".to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

    Ok(Value::Map(resp))
}

/// server/no-content - 204 No Contentレスポンスを作成
pub fn native_server_no_content(_args: &[Value]) -> Result<Value, String> {
    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(204),
    );
    resp.insert(
        Value::Keyword("body".to_string()).to_map_key().unwrap(),
        Value::String(String::new()),
    );
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
        crate::new_hashmap()
    };

    let port_key = Value::Keyword("port".to_string()).to_map_key().unwrap();
    let host_key = Value::Keyword("host".to_string()).to_map_key().unwrap();
    let timeout_key = Value::Keyword("timeout".to_string()).to_map_key().unwrap();

    let port = match opts.get(&port_key) {
        Some(Value::Integer(p)) => *p as u16,
        _ => 3000,
    };

    let host = match opts.get(&host_key) {
        Some(Value::String(h)) => h.clone(),
        _ => "127.0.0.1".to_string(),
    };

    let timeout_secs = match opts.get(&timeout_key) {
        Some(Value::Integer(t)) => *t as u64,
        _ => 30,
    };

    println!(
        "HTTP server started on http://{}:{} (timeout: {}s)",
        host, port, timeout_secs
    );

    // ブロッキングで実行（Ctrl+Cでシャットダウンするまで待つ）
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToCreateRuntime, &[&e.to_string()]))?;

    rt.block_on(async move {
        if let Err(e) = run_server(&host, port, handler, timeout_secs).await {
            eprintln!("Server error: {}", e);
        }
    });

    Ok(Value::Nil)
}

/// サーバー実行
async fn run_server(
    host: &str,
    port: u16,
    handler: Value,
    timeout_secs: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    // ハンドラーをArcで共有
    let handler = Arc::new(handler);
    let timeout_duration = Duration::from_secs(timeout_secs);

    // グレースフルシャットダウン用のシグナル
    let shutdown = Arc::new(tokio::sync::Notify::new());
    let shutdown_clone = shutdown.clone();

    // シグナルハンドラー（Ctrl+C と SIGTERM）
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("failed to create SIGTERM signal handler");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("failed to create SIGINT signal handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    println!("\nReceived SIGTERM, gracefully shutting down...");
                }
                _ = sigint.recv() => {
                    println!("\nReceived SIGINT (Ctrl+C), gracefully shutting down...");
                }
            }
        }
        #[cfg(not(unix))]
        {
            // Windows環境ではCtrl+Cのみ対応
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    println!("\nReceived shutdown signal, gracefully shutting down...");
                }
                Err(err) => {
                    eprintln!("Error listening for shutdown signal: {}", err);
                    return;
                }
            }
        }
        shutdown_clone.notify_waiters();
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
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
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
            _ => Err(fmt_msg(
                MsgKey::ServerHandlerMustBeFunction,
                &[handler.type_name()],
            )),
        };

        // Qi値をHTTPレスポンスに変換（async）
        match resp_value {
            Ok(v) => value_to_response(v).await,
            Err(e) => {
                eprintln!("Handler error: {}", e);
                Err(fmt_msg(MsgKey::ServerHandlerError, &[&e]))
            }
        }
    })
    .await;

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
fn route_request(req: &Value, routes: &im::Vector<Value>) -> Result<Value, String> {
    let method_key = Value::Keyword("method".to_string()).to_map_key().unwrap();
    let path_key = Value::Keyword("path".to_string()).to_map_key().unwrap();

    let method = match req {
        Value::Map(m) => match m.get(&method_key) {
            Some(Value::Keyword(k)) => k.clone(),
            _ => {
                return Err(fmt_msg(
                    MsgKey::RequestMustHave,
                    &["request", ":method keyword"],
                ))
            }
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    let path = match req {
        Value::Map(m) => match m.get(&path_key) {
            Some(Value::String(p)) => p.clone(),
            _ => {
                return Err(fmt_msg(
                    MsgKey::RequestMustHave,
                    &["request", ":path string"],
                ))
            }
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    // ルートを探索
    for route in routes {
        if let Value::Vector(route_def) = route {
            if route_def.len() == 2 {
                if let (Value::String(pattern), Value::Map(handlers)) =
                    (&route_def[0], &route_def[1])
                {
                    // メソッドに対応するハンドラーを取得
                    if let Some(handler) = handlers.get(&method) {
                        // 静的ファイルハンドラーの場合はプレフィックスマッチング
                        if let Value::Map(m) = handler {
                            if m.contains_key("__static_dir__") {
                                // プレフィックスマッチング（パスがパターンで始まっているか）
                                let pattern_normalized = if pattern == "/" {
                                    "/"
                                } else {
                                    pattern.trim_end_matches('/')
                                };
                                let path_normalized = path.trim_end_matches('/');

                                if path_normalized == pattern_normalized
                                    || (pattern_normalized == "/" && !path.is_empty())
                                    || path.starts_with(&format!("{}/", pattern_normalized))
                                {
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
                                _ => crate::new_hashmap(),
                            };
                            let params_key =
                                Value::Keyword("params".to_string()).to_map_key().unwrap();
                            req_with_params.insert(params_key, Value::Map(params));

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

/// 静的ファイルを配信（ストリーミング対応）
fn serve_static_file(dir_path: &str, req: &Value) -> Result<Value, String> {
    let path_key = Value::Keyword("path".to_string()).to_map_key().unwrap();

    let path = match req {
        Value::Map(m) => match m.get(&path_key) {
            Some(Value::String(p)) => p,
            _ => {
                return Err(fmt_msg(
                    MsgKey::RequestMustHave,
                    &["request", ":path string"],
                ))
            }
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

    // ファイルの存在確認（メタデータ取得）
    std::fs::metadata(&file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            return format!("File not found: {}", path);
        }
        format!("Failed to read file metadata: {}", e)
    })?;

    // ストリーミングレスポンスを生成（:body-file を使用）
    let content_type = get_content_type(file_path.to_str().unwrap_or(""));
    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| fmt_msg(MsgKey::InvalidFilePath, &["serve_static_file"]))?;

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(200),
    );
    resp.insert(
        Value::Keyword("body-file".to_string())
            .to_map_key()
            .unwrap(),
        Value::String(file_path_str.to_string()),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String(content_type.to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

    Ok(Value::Map(resp))
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
                            let headers_key =
                                Value::Keyword("headers".to_string()).to_map_key().unwrap();
                            let auth_header = req_map
                                .get(&headers_key)
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
                                    use base64::{engine::general_purpose, Engine as _};
                                    let encoded = auth.strip_prefix("Basic ").unwrap(); // "Basic " を除く
                                    if let Ok(decoded_bytes) =
                                        general_purpose::STANDARD.decode(encoded)
                                    {
                                        if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                                            if let Some((user, pass)) = decoded.split_once(':') {
                                                // ユーザー名とパスワードを検証
                                                users
                                                    .get(user)
                                                    .and_then(|v| match v {
                                                        Value::String(expected_pass) => {
                                                            Some(pass == expected_pass)
                                                        }
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
                            let mut resp = crate::new_hashmap();
                            resp.insert(
                                Value::Keyword("status".to_string()).to_map_key().unwrap(),
                                Value::Integer(401),
                            );
                            resp.insert(
                                Value::Keyword("body".to_string()).to_map_key().unwrap(),
                                Value::String("Unauthorized".to_string()),
                            );
                            let mut headers = crate::new_hashmap();
                            headers.insert(
                                "WWW-Authenticate".to_string(),
                                Value::String("Basic realm=\"Restricted\"".to_string()),
                            );
                            resp.insert(
                                Value::Keyword("headers".to_string()).to_map_key().unwrap(),
                                Value::Map(headers),
                            );
                            return Ok(Value::Map(resp));
                        }
                    }
                }

                // リクエストを前処理（json-body, bearer）
                let processed_req = match middleware_type.as_str() {
                    "json-body" => apply_json_body_middleware(req),
                    "bearer" => apply_bearer_middleware(req),
                    _ => req.clone(),
                };

                // ロギング（リクエスト）
                if middleware_type == "logging" {
                    apply_logging_middleware(&processed_req);
                }

                // 内部ハンドラーを再帰的に実行（ネストしたミドルウェア対応）
                let response = apply_middleware(inner_handler, &processed_req, eval)?;

                // レスポンスを後処理（cors, compression, logging）
                let processed_resp = match middleware_type.as_str() {
                    "cors" => {
                        let origins = m
                            .get("__origins__")
                            .and_then(|v| match v {
                                Value::Vector(v) => Some(v.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| vec![Value::String("*".to_string())].into());
                        apply_cors_middleware(&response, &origins)
                    }
                    "compression" => {
                        let min_size = m
                            .get("__min_size__")
                            .and_then(|v| match v {
                                Value::Integer(s) => Some(*s as usize),
                                _ => None,
                            })
                            .unwrap_or(1024);
                        apply_compression_middleware(&response, min_size)
                    }
                    "logging" => {
                        // レスポンスステータスをログ出力
                        if let Value::Map(resp_map) = &response {
                            let status_key =
                                Value::Keyword("status".to_string()).to_map_key().unwrap();
                            let status = resp_map
                                .get(&status_key)
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
                            let headers_key =
                                Value::Keyword("headers".to_string()).to_map_key().unwrap();
                            let mut headers = match resp_map.get(&headers_key) {
                                Some(Value::Map(h)) => h.clone(),
                                _ => crate::new_hashmap(),
                            };

                            headers.insert(
                                "Cache-Control".to_string(),
                                Value::String(
                                    "no-store, no-cache, must-revalidate, private".to_string(),
                                ),
                            );
                            headers.insert(
                                "Pragma".to_string(),
                                Value::String("no-cache".to_string()),
                            );
                            headers.insert("Expires".to_string(), Value::String("0".to_string()));

                            resp_map.insert(headers_key, Value::Map(headers));
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

                                let max_age_key =
                                    Value::Keyword("max-age".to_string()).to_map_key().unwrap();
                                let public_key =
                                    Value::Keyword("public".to_string()).to_map_key().unwrap();
                                let private_key =
                                    Value::Keyword("private".to_string()).to_map_key().unwrap();
                                let no_store_key =
                                    Value::Keyword("no-store".to_string()).to_map_key().unwrap();
                                let must_revalidate_key =
                                    Value::Keyword("must-revalidate".to_string())
                                        .to_map_key()
                                        .unwrap();
                                let immutable_key = Value::Keyword("immutable".to_string())
                                    .to_map_key()
                                    .unwrap();

                                // max-age
                                if let Some(Value::Integer(age)) = opts.get(&max_age_key) {
                                    cache_parts.push(format!("max-age={}", age));
                                }

                                // public/private
                                if let Some(Value::Bool(true)) = opts.get(&public_key) {
                                    cache_parts.push("public".to_string());
                                } else if let Some(Value::Bool(true)) = opts.get(&private_key) {
                                    cache_parts.push("private".to_string());
                                }

                                // no-store
                                if let Some(Value::Bool(true)) = opts.get(&no_store_key) {
                                    cache_parts.push("no-store".to_string());
                                }

                                // must-revalidate
                                if let Some(Value::Bool(true)) = opts.get(&must_revalidate_key) {
                                    cache_parts.push("must-revalidate".to_string());
                                }

                                // immutable
                                if let Some(Value::Bool(true)) = opts.get(&immutable_key) {
                                    cache_parts.push("immutable".to_string());
                                }

                                if !cache_parts.is_empty() {
                                    let cache_control = cache_parts.join(", ");
                                    let headers_key =
                                        Value::Keyword("headers".to_string()).to_map_key().unwrap();
                                    let mut headers = match resp_map.get(&headers_key) {
                                        Some(Value::Map(h)) => h.clone(),
                                        _ => crate::new_hashmap(),
                                    };
                                    headers.insert(
                                        "Cache-Control".to_string(),
                                        Value::String(cache_control),
                                    );
                                    resp_map.insert(headers_key, Value::Map(headers));
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
    eval.apply_function(handler, std::slice::from_ref(req))
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

    let mut params = crate::new_hashmap();

    // 各パートを比較
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if pattern_part.starts_with(':') {
            // パラメータ部分 - パラメータ名を抽出
            let param_name = pattern_part.strip_prefix(':').unwrap(); // ':' を除く
            params.insert(param_name.to_string(), Value::String(path_part.to_string()));
        } else if pattern_part != path_part {
            // 固定部分が一致しない
            return None;
        }
    }

    Some(params)
}

/// エラーレスポンス生成
fn error_response(status: u16, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Full::new(Bytes::from(message.to_string())).boxed())
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
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("logging".to_string()),
    );
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
            Value::Map(m) => {
                let origins_key = Value::Keyword("origins".to_string()).to_map_key().unwrap();
                match m.get(&origins_key) {
                    Some(Value::Vector(v)) => v
                        .iter()
                        .filter_map(|val| match val {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect(),
                    _ => vec!["*".to_string()],
                }
            }
            _ => vec!["*".to_string()],
        }
    } else {
        vec!["*".to_string()]
    };

    // CORSミドルウェアマーカーとして、マップにメタデータを埋め込む
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("cors".to_string()),
    );
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert(
        "__origins__".to_string(),
        Value::Vector(origins.iter().map(|s| Value::String(s.clone())).collect()),
    );

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
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("json-body".to_string()),
    );
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
            Value::Map(m) => {
                let min_size_key = Value::Keyword("min-size".to_string()).to_map_key().unwrap();
                match m.get(&min_size_key) {
                    Some(Value::Integer(s)) => *s as usize,
                    _ => 1024, // デフォルト: 1KB以上で圧縮
                }
            }
            _ => 1024,
        }
    } else {
        1024
    };

    // 圧縮ミドルウェアマーカー
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("compression".to_string()),
    );
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

/// server/static-file - 単一ファイルを配信するレスポンスを生成（ストリーミング対応）
pub fn native_server_static_file(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-file"]));
    }

    let file_path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["server/static-file", "file path"],
            ))
        }
    };

    // セキュリティチェック
    if !is_safe_path(file_path) {
        return Err(fmt_msg(MsgKey::InvalidFilePath, &["server/static-file"]));
    }

    // ファイルの存在確認
    std::fs::metadata(file_path)
        .map_err(|e| fmt_msg(MsgKey::ServerStaticFileMetadataFailed, &[&e.to_string()]))?;

    // ストリーミングレスポンスを生成（:body-file を使用）
    let content_type = get_content_type(file_path);

    let mut resp = crate::new_hashmap();
    resp.insert(
        Value::Keyword("status".to_string()).to_map_key().unwrap(),
        Value::Integer(200),
    );
    resp.insert(
        Value::Keyword("body-file".to_string())
            .to_map_key()
            .unwrap(),
        Value::String(file_path.to_string()),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        "Content-Type".to_string(),
        Value::String(content_type.to_string()),
    );
    resp.insert(
        Value::Keyword("headers".to_string()).to_map_key().unwrap(),
        Value::Map(headers),
    );

    Ok(Value::Map(resp))
}

/// server/static-dir - ディレクトリから静的ファイルを配信するハンドラーを生成
pub fn native_server_static_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-dir"]));
    }

    let dir_path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["server/static-dir", "directory path"],
            ))
        }
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
    let mut metadata = crate::new_hashmap();
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
            Value::Map(m) => {
                let users_key = Value::Keyword("users".to_string()).to_map_key().unwrap();
                match m.get(&users_key) {
                    Some(Value::Map(u)) => u.clone(),
                    _ => crate::new_hashmap(),
                }
            }
            _ => crate::new_hashmap(),
        }
    } else {
        crate::new_hashmap()
    };

    // Basic Authミドルウェアマーカー
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("basic-auth".to_string()),
    );
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
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("bearer".to_string()),
    );
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
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("no-cache".to_string()),
    );
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
            _ => crate::new_hashmap(),
        }
    } else {
        crate::new_hashmap()
    };

    // cache-controlミドルウェアマーカー
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        "__middleware__".to_string(),
        Value::String("cache-control".to_string()),
    );
    metadata.insert("__handler__".to_string(), handler);
    metadata.insert("__cache_opts__".to_string(), Value::Map(opts));

    Ok(Value::Map(metadata))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category server
/// @qi-doc:functions serve, router, ok, json, response, not-found, no-content, with-logging, with-cors, with-json-body, static-file, static-dir
pub const FUNCTIONS: super::NativeFunctions = &[
    ("server/serve", native_server_serve),
    ("server/router", native_server_router),
    ("server/ok", native_server_ok),
    ("server/json", native_server_json),
    ("server/response", native_server_response),
    ("server/not-found", native_server_not_found),
    ("server/no-content", native_server_no_content),
    ("server/with-logging", native_server_with_logging),
    ("server/with-cors", native_server_with_cors),
    ("server/with-json-body", native_server_with_json_body),
    ("server/with-compression", native_server_with_compression),
    ("server/with-basic-auth", native_server_with_basic_auth),
    ("server/with-bearer", native_server_with_bearer),
    ("server/with-no-cache", native_server_with_no_cache),
    (
        "server/with-cache-control",
        native_server_with_cache_control,
    ),
    ("server/static-file", native_server_static_file),
    ("server/static-dir", native_server_static_dir),
];
