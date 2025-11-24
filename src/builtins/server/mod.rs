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

// ========================================
// 公開定数
// ========================================

/// デフォルトHTTPサーバーポート
pub const DEFAULT_HTTP_PORT: u16 = 3000;

/// デフォルトHTTPバインドホスト
pub const DEFAULT_HTTP_HOST: &str = "127.0.0.1";

/// デフォルトHTTPサーバータイムアウト（秒）
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

// ========================================
// サブモジュール
// ========================================

mod helpers;
mod middleware;
mod response;
mod routing;
mod serve;
mod static_files;

// 公開エクスポート
pub use middleware::*;
pub use response::*;
pub use routing::native_server_router;
pub use serve::native_server_serve;
pub use static_files::{native_server_static_dir, native_server_static_file};

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
