//! サーバーヘルパー関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use crate::HashMap;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::body::{Bytes, Frame};
use hyper::{Request, Response};
use std::convert::Infallible;
use std::io::Read;
use tokio::fs::File as TokioFile;
use tokio_util::io::ReaderStream;

/// キーワードをマップキーに変換するヘルパー関数
/// SAFETY: キーワード文字列は常に有効なマップキーに変換できる
#[inline]
pub(super) fn kw(s: &str) -> String {
    Value::Keyword(s.to_string())
        .to_map_key()
        .expect("Keyword should always convert to MapKey")
}

pub(super) fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// gzip圧縮ヘルパー関数（レスポンス用）
/// UTF-8文字列を圧縮し、圧縮されたバイナリデータをバイト列として表現したStringとして返す
pub(super) fn compress_gzip_response(body: &str) -> Result<String, std::io::Error> {
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

pub(super) fn parse_query_params(query_str: &str) -> HashMap<String, Value> {
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
pub(super) async fn request_to_value(
    req: Request<hyper::body::Incoming>,
) -> Result<(Value, String), String> {
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
pub(super) async fn create_file_stream_body(
    file_path: &str,
) -> Result<BoxBody<Bytes, Infallible>, String> {
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
    Ok(BodyExt::boxed(body))
}

/// Qi値をHTTPレスポンスに変換
pub(super) async fn value_to_response(
    value: Value,
) -> Result<Response<BoxBody<Bytes, Infallible>>, String> {
    match value {
        Value::Map(m) => {
            // {:status 200, :headers {...}, :body "..." or :body-file "/path"}
            let status_key = kw("status");
            let headers_key = kw("headers");
            let body_key = kw("body");
            let body_file_key = kw("body-file");

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

                    BodyExt::boxed(Full::new(Bytes::from(body_bytes)))
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

/// 安全なパス検証（ベースディレクトリ外へのアクセスを防止）
///
/// パストラバーサル攻撃を防ぐため、以下のチェックを実施:
/// - URLデコード後のパス検証
/// - 正規化（canonicalize）によるシンボリックリンク・..の解決

pub(super) fn error_response(status: u16, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    // SAFETY: Response::builderは有効なステータスコードとヘッダーでは失敗しない
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(BodyExt::boxed(Full::new(Bytes::from(message.to_string()))))
        .expect("Failed to build HTTP response")
}

// ========================================
