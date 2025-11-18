//! Redis接続モジュール
//!
//! このモジュールは `kvs-redis` feature でコンパイルされます。

use dashmap::DashMap;
use redis::aio::MultiplexedConnection;
use redis::Client;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Redis接続プール（URL → Connection のマッピング）
pub(super) static REDIS_POOL: LazyLock<DashMap<String, MultiplexedConnection>> =
    LazyLock::new(DashMap::new);

/// グローバルなtokioランタイム（Redisの非同期操作用）
pub(super) static TOKIO_RT: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime for Redis"));

/// 接続を取得または新規作成
pub(super) async fn get_or_create_connection(url: &str) -> Result<MultiplexedConnection, String> {
    // 既存の接続を取得
    if let Some(conn) = REDIS_POOL.get(url) {
        return Ok(conn.clone());
    }

    // 新規接続を作成
    let client = Client::open(url).map_err(|e| format!("Connection error: {}", e))?;

    let conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    // プールに保存
    REDIS_POOL.insert(url.to_string(), conn.clone());

    Ok(conn)
}

/// 接続エラー時にプールから削除して再接続
pub(super) async fn reconnect(url: &str) -> Result<MultiplexedConnection, String> {
    // プールから古い接続を削除
    REDIS_POOL.remove(url);

    // 新規接続を作成
    get_or_create_connection(url).await
}

/// Redis操作を再試行付きで実行するヘルパー
pub(super) async fn execute_with_retry<T, F, Fut>(url: &str, operation: F) -> redis::RedisResult<T>
where
    F: Fn(MultiplexedConnection) -> Fut,
    Fut: std::future::Future<Output = redis::RedisResult<T>>,
{
    // 最初の試行
    let conn = get_or_create_connection(url)
        .await
        .map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "Connection error", e)))?;

    let result = operation(conn).await;

    // エラーの場合、再接続して再試行
    if let Err(ref e) = result {
        let err_str = e.to_string();
        if err_str.contains("broken pipe")
            || err_str.contains("Connection")
            || err_str.contains("terminated")
        {
            // 再接続
            if let Ok(new_conn) = reconnect(url).await {
                return operation(new_conn).await;
            }
        }
    }

    result
}
