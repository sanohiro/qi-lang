//! Lazy初期化サポート
//!
//! 各モジュールは初回使用時にのみ初期化される。
//! Lazy Initにより、未使用機能のメモリ消費をゼロに抑える。

use once_cell::sync::{Lazy, OnceCell};

/// HTTPクライアントのLazy初期化
#[cfg(feature = "http-client")]
pub mod http_client {
    use super::*;

    pub static CLIENT: Lazy<Result<reqwest::blocking::Client, String>> = Lazy::new(|| {
        reqwest::blocking::Client::builder()
            .user_agent("qi-lang/0.1.0")
            .gzip(true)
            .deflate(true)
            .brotli(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))
    });

    /// HTTPクライアントを取得（エラーハンドリング付き）
    pub fn get_client() -> Result<&'static reqwest::blocking::Client, String> {
        CLIENT.as_ref().map_err(|e| e.clone())
    }
}

/// HTTPサーバーランタイムのLazy初期化
#[cfg(feature = "http-server")]
pub mod http_server {
    use super::*;

    pub static RUNTIME: OnceCell<Result<tokio::runtime::Runtime, String>> = OnceCell::new();

    /// サーバーランタイムを取得（初回のみ作成、エラーハンドリング付き）
    pub fn get_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
        let result = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(4)
                .thread_name("qi-server")
                .build()
                .map_err(|e| format!("Failed to create server runtime: {}", e))
        });

        result.as_ref().map_err(|e| e.clone())
    }
}
