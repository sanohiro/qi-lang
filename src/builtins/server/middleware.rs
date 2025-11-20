//! ミドルウェア関数

use super::helpers::{compress_gzip_response, kw};
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

pub(super) fn apply_json_body_middleware(req: &Value) -> Value {
    let Value::Map(req_map) = req else {
        return req.clone();
    };

    let body_key = kw("body");
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

    let ok_key = kw("ok");
    let Some(json_value) = result.get(&ok_key) else {
        return req.clone();
    };

    // JSON解析成功 → 変更が必要なのでclone
    let mut new_req = req_map.clone();
    let json_key = kw("json");
    new_req.insert(json_key, json_value.clone());
    Value::Map(new_req)
}

/// Bearerトークンを抽出してリクエストに追加
pub(super) fn apply_bearer_middleware(req: &Value) -> Value {
    let Value::Map(req_map) = req else {
        return req.clone();
    };

    let headers_key = kw("headers");

    let token = req_map
        .get(&headers_key)
        .and_then(|h| match h {
            Value::Map(headers) => {
                headers.get(&crate::value::MapKey::String("authorization".to_string()))
            }
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
    let bearer_key = kw("bearer-token");
    new_req.insert(bearer_key, Value::String(token.to_string()));
    Value::Map(new_req)
}

/// リクエストをロギング
pub(super) fn apply_logging_middleware(req: &Value) {
    if let Value::Map(req_map) = req {
        let method_key = kw("method");
        let path_key = kw("path");

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
pub(super) fn apply_cors_middleware(resp: &Value, origins: &im::Vector<Value>) -> Value {
    let Value::Map(resp_map) = resp else {
        return resp.clone();
    };

    let headers_key = kw("headers");

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
        crate::value::MapKey::String("Access-Control-Allow-Origin".to_string()),
        Value::String(origin),
    );
    headers.insert(
        crate::value::MapKey::String("Access-Control-Allow-Methods".to_string()),
        Value::String("GET, POST, PUT, DELETE, OPTIONS".to_string()),
    );
    headers.insert(
        crate::value::MapKey::String("Access-Control-Allow-Headers".to_string()),
        Value::String("Content-Type, Authorization".to_string()),
    );

    // CORSヘッダー追加 → 変更が必要なのでclone
    let mut new_resp = resp_map.clone();
    new_resp.insert(headers_key, Value::Map(headers));
    Value::Map(new_resp)
}

/// レスポンスボディを圧縮
pub(super) fn apply_compression_middleware(resp: &Value, min_size: usize) -> Value {
    let Value::Map(resp_map) = resp else {
        return resp.clone();
    };

    let body_key = kw("body");
    let headers_key = kw("headers");

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
        crate::value::MapKey::String("Content-Encoding".to_string()),
        Value::String("gzip".to_string()),
    );
    new_resp.insert(headers_key.clone(), Value::Map(headers));
    new_resp.insert(body_key, Value::String(compressed));
    Value::Map(new_resp)
}

pub fn native_server_with_logging(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/with-logging"]));
    }

    let handler = args[0].clone();

    // ロギングミドルウェアマーカー
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("logging".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );

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
                let origins_key = kw("origins");
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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("cors".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );
    metadata.insert(
        crate::value::MapKey::String("__origins__".to_string()),
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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("json-body".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );

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
                let min_size_key = kw("min-size");
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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("compression".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );
    metadata.insert(
        crate::value::MapKey::String("__min_size__".to_string()),
        Value::Integer(min_size as i64),
    );

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
                let users_key = kw("users");
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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("basic-auth".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );
    metadata.insert(
        crate::value::MapKey::String("__users__".to_string()),
        Value::Map(users),
    );

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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("bearer".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );

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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("no-cache".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );

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
        crate::value::MapKey::String("__middleware__".to_string()),
        Value::String("cache-control".to_string()),
    );
    metadata.insert(
        crate::value::MapKey::String("__handler__".to_string()),
        handler,
    );
    metadata.insert(
        crate::value::MapKey::String("__cache_opts__".to_string()),
        Value::Map(opts),
    );

    Ok(Value::Map(metadata))
}
