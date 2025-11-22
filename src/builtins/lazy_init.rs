//! Lazy初期化サポート
//!
//! 各モジュールは初回使用時にのみ初期化される。
//! Lazy Initにより、未使用機能のメモリ消費をゼロに抑える。

use once_cell::sync::{Lazy, OnceCell};

/// HTTPクライアントのLazy初期化
#[cfg(feature = "http-client")]
pub mod http_client {
    use super::*;

    pub static CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(|| {
        reqwest::blocking::Client::builder()
            .user_agent("qi-lang/0.1.0")
            .gzip(true)
            .deflate(true)
            .brotli(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Fatal error: Failed to create HTTP client: {}", e);
                eprintln!("This is a critical initialization error. The program cannot continue.");
                std::process::exit(1);
            })
    });
}

/// HTTPサーバーランタイムのLazy初期化
#[cfg(feature = "http-server")]
pub mod http_server {
    use super::*;

    pub static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

    /// サーバーランタイムを取得（初回のみ作成）
    pub fn get_runtime() -> &'static tokio::runtime::Runtime {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(4)
                .thread_name("qi-server")
                .build()
                .unwrap_or_else(|e| {
                    eprintln!("Fatal error: Failed to create server runtime: {}", e);
                    eprintln!(
                        "This is a critical initialization error. The program cannot continue."
                    );
                    std::process::exit(1);
                })
        })
    }
}
