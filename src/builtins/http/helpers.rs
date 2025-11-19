use super::*;

/// gzip圧縮ヘルパー関数
pub(super) fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// オプションMapからヘッダーとタイムアウトを抽出する
/// 引数: オプションMap（Option<&Value>）
/// 戻り値: (ヘッダーMap, タイムアウトms)
pub(super) fn parse_http_options(
    opts: Option<&Value>,
) -> Result<(Option<crate::HashMap<String, Value>>, u64), String> {
    let Some(Value::Map(opts_map)) = opts else {
        // オプションがない場合はデフォルト値
        return Ok((None, 30000));
    };

    // キーを準備
    let headers_key = Value::Keyword(crate::intern::intern_keyword("headers"))
        .to_map_key()
        .unwrap_or_else(|_| "headers".to_string());
    let basic_auth_key = Value::Keyword(crate::intern::intern_keyword("basic-auth"))
        .to_map_key()
        .unwrap_or_else(|_| "basic-auth".to_string());
    let bearer_token_key = Value::Keyword(crate::intern::intern_keyword("bearer-token"))
        .to_map_key()
        .unwrap_or_else(|_| "bearer-token".to_string());
    let timeout_key = Value::Keyword(crate::intern::intern_keyword("timeout"))
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
