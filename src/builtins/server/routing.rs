//! ルーティング機能

use super::helpers::kw;
use super::middleware::{
    apply_bearer_middleware, apply_compression_middleware, apply_cors_middleware,
    apply_json_body_middleware, apply_logging_middleware,
};
use super::response::native_server_not_found;
use super::static_files::serve_static_file;
use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

// HTTPヘッダー定数
const HEADER_CACHE_CONTROL: &str = "Cache-Control";

// Cache-Controlディレクティブ定数
const CACHE_DIRECTIVE_PUBLIC: &str = "public";
const CACHE_DIRECTIVE_PRIVATE: &str = "private";
const CACHE_DIRECTIVE_NO_STORE: &str = "no-store";
const CACHE_DIRECTIVE_MUST_REVALIDATE: &str = "must-revalidate";
const CACHE_DIRECTIVE_IMMUTABLE: &str = "immutable";

pub fn native_server_router(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["server/router", "1"]));
    }

    // ルート定義をそのまま返す（後でserveで使用）
    // ルートは [[path {:get handler, :post handler}], ...] の形式
    Ok(args[0].clone())
}

/// server/serve - HTTPサーバーを起動
///
/// ## オプション
///
/// - `:timeout` - リクエストタイムアウト（秒）
///   - 最小: 1秒
///   - 最大: 300秒（5分）
///   - デフォルト: 30秒
pub(super) fn route_request(req: &Value, routes: &im::Vector<Value>) -> Result<Value, String> {
    let method_key = kw("method");
    let path_key = kw("path");

    let method = match req {
        Value::Map(m) => match m.get(&method_key) {
            Some(Value::Keyword(k)) => k.to_string(),
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
                    if let Some(handler) =
                        handlers.get(&crate::value::MapKey::String(method.clone()))
                    {
                        // 静的ファイルハンドラーの場合はプレフィックスマッチング
                        if let Value::Map(m) = handler {
                            if m.contains_key(&crate::value::MapKey::String(
                                "__static_dir__".to_string(),
                            )) {
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
                            let params_key = kw("params");
                            req_with_params.insert(
                                params_key,
                                Value::Map(crate::builtins::util::convert_string_map_to_mapkey(
                                    params,
                                )),
                            );

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

/// ミドルウェアを適用してハンドラーを実行
pub(super) fn apply_middleware(
    handler: &Value,
    req: &Value,
    eval: &Evaluator,
) -> Result<Value, String> {
    // 静的ファイルハンドラーかチェック
    if let Value::Map(m) = handler {
        if let Some(Value::String(dir_path)) =
            m.get(&crate::value::MapKey::String("__static_dir__".to_string()))
        {
            // 静的ファイル配信
            return serve_static_file(dir_path, req);
        }

        if let Some(Value::String(middleware_type)) =
            m.get(&crate::value::MapKey::String("__middleware__".to_string()))
        {
            // ミドルウェアの場合、内部のハンドラーを取得
            if let Some(inner_handler) =
                m.get(&crate::value::MapKey::String("__handler__".to_string()))
            {
                // Basic Auth検証
                if middleware_type == "basic-auth" {
                    if let Value::Map(req_map) = req {
                        let authorized = if let Some(Value::Map(users)) =
                            m.get(&crate::value::MapKey::String("__users__".to_string()))
                        {
                            // Authorizationヘッダーを取得
                            let headers_key = kw("headers");
                            let auth_header = req_map
                                .get(&headers_key)
                                .and_then(|h| match h {
                                    Value::Map(headers) => headers.get(
                                        &crate::value::MapKey::String("authorization".to_string()),
                                    ),
                                    _ => None,
                                })
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                });

                            if let Some(auth) = auth_header {
                                if let Some(encoded) = auth.strip_prefix("Basic ") {
                                    use base64::{engine::general_purpose, Engine as _};
                                    if let Ok(decoded_bytes) =
                                        general_purpose::STANDARD.decode(encoded)
                                    {
                                        if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                                            if let Some((user, pass)) = decoded.split_once(':') {
                                                // ユーザー名とパスワードを検証
                                                users
                                                    .get(&crate::value::MapKey::String(
                                                        user.to_string(),
                                                    ))
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
                            resp.insert(kw("status"), Value::Integer(401));
                            resp.insert(kw("body"), Value::String("Unauthorized".to_string()));
                            let mut headers = crate::new_hashmap();
                            headers.insert(
                                crate::value::MapKey::String("WWW-Authenticate".to_string()),
                                Value::String("Basic realm=\"Restricted\"".to_string()),
                            );
                            resp.insert(kw("headers"), Value::Map(headers));
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
                            .get(&crate::value::MapKey::String("__origins__".to_string()))
                            .and_then(|v| match v {
                                Value::Vector(v) => Some(v.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| vec![Value::String("*".to_string())].into());
                        apply_cors_middleware(&response, &origins)
                    }
                    "compression" => {
                        let min_size = m
                            .get(&crate::value::MapKey::String("__min_size__".to_string()))
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
                            let status_key = kw("status");
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
                            let headers_key = kw("headers");
                            let mut headers = match resp_map.get(&headers_key) {
                                Some(Value::Map(h)) => h.clone(),
                                _ => crate::new_hashmap(),
                            };

                            headers.insert(
                                crate::value::MapKey::String(HEADER_CACHE_CONTROL.to_string()),
                                Value::String(
                                    "no-store, no-cache, must-revalidate, private".to_string(),
                                ),
                            );
                            headers.insert(
                                crate::value::MapKey::String("Pragma".to_string()),
                                Value::String("no-cache".to_string()),
                            );
                            headers.insert(
                                crate::value::MapKey::String("Expires".to_string()),
                                Value::String("0".to_string()),
                            );

                            resp_map.insert(headers_key, Value::Map(headers));
                            Value::Map(resp_map)
                        } else {
                            response
                        }
                    }
                    "cache-control" => {
                        // カスタムCache-Controlヘッダーを追加
                        if let Value::Map(mut resp_map) = response.clone() {
                            if let Some(Value::Map(opts)) =
                                m.get(&crate::value::MapKey::String("__cache_opts__".to_string()))
                            {
                                let mut cache_parts = Vec::new();

                                let max_age_key = kw("max-age");
                                let public_key = kw("public");
                                let private_key = kw("private");
                                let no_store_key = kw("no-store");
                                let must_revalidate_key = kw("must-revalidate");
                                let immutable_key = kw("immutable");

                                // max-age
                                if let Some(Value::Integer(age)) = opts.get(&max_age_key) {
                                    cache_parts.push(format!("max-age={}", age));
                                }

                                // public/private
                                if let Some(Value::Bool(true)) = opts.get(&public_key) {
                                    cache_parts.push(CACHE_DIRECTIVE_PUBLIC.to_string());
                                } else if let Some(Value::Bool(true)) = opts.get(&private_key) {
                                    cache_parts.push(CACHE_DIRECTIVE_PRIVATE.to_string());
                                }

                                // no-store
                                if let Some(Value::Bool(true)) = opts.get(&no_store_key) {
                                    cache_parts.push(CACHE_DIRECTIVE_NO_STORE.to_string());
                                }

                                // must-revalidate
                                if let Some(Value::Bool(true)) = opts.get(&must_revalidate_key) {
                                    cache_parts.push(CACHE_DIRECTIVE_MUST_REVALIDATE.to_string());
                                }

                                // immutable
                                if let Some(Value::Bool(true)) = opts.get(&immutable_key) {
                                    cache_parts.push(CACHE_DIRECTIVE_IMMUTABLE.to_string());
                                }

                                if !cache_parts.is_empty() {
                                    let cache_control = cache_parts.join(", ");
                                    let headers_key = kw("headers");
                                    let mut headers = match resp_map.get(&headers_key) {
                                        Some(Value::Map(h)) => h.clone(),
                                        _ => crate::new_hashmap(),
                                    };
                                    headers.insert(
                                        crate::value::MapKey::String(
                                            HEADER_CACHE_CONTROL.to_string(),
                                        ),
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
fn match_route_pattern(
    pattern: &str,
    path: &str,
) -> Option<std::collections::HashMap<String, Value>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // パート数が異なる場合はマッチしない
    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = std::collections::HashMap::new();

    // 各パートを比較
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(param_name) = pattern_part.strip_prefix(':') {
            // パラメータ部分 - パラメータ名を抽出
            params.insert(param_name.to_string(), Value::String(path_part.to_string()));
        } else if pattern_part != path_part {
            // 固定部分が一致しない
            return None;
        }
    }

    Some(params)
}
