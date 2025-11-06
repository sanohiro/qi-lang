//! WebSocket機能
//!
//! Pure RustのWebSocketサーバー/クライアント実装（tokio-tungstenite）
//!
//! ## クライアントサイド
//! - `ws/connect` - WebSocket接続（接続IDを返す）
//! - `ws/send` - メッセージ送信
//! - `ws/receive` - メッセージ受信
//! - `ws/close` - 接続クローズ

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};
use tokio::sync::Mutex;

/// ネイティブ関数の型エイリアス
type NativeFn = fn(&[Value]) -> Result<Value, String>;

/// @qi-doc:category net/websocket
/// @qi-doc:functions ws/connect, ws/send, ws/receive, ws/close
/// @qi-doc:note WebSocket通信（クライアント）
pub const FUNCTIONS: &[(&str, NativeFn)] = &[
    ("ws/connect", native_ws_connect as NativeFn),
    ("ws/send", native_ws_send as NativeFn),
    ("ws/receive", native_ws_receive as NativeFn),
    ("ws/close", native_ws_close as NativeFn),
];

#[cfg(feature = "websocket")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "websocket")]
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::CloseFrame, Message},
    MaybeTlsStream, WebSocketStream,
};

// ========================================
// WebSocket接続の管理
// ========================================

/// WebSocket接続（IDベース管理）
pub struct WebSocketConnection {
    stream: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>,
}

impl WebSocketConnection {
    /// 新しい接続を作成
    pub fn new(stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        Self {
            stream: Arc::new(Mutex::new(Some(stream))),
        }
    }

    /// メッセージを送信
    pub async fn send(&self, data: &str) -> Result<(), String> {
        let mut stream_guard = self.stream.lock().await;
        if let Some(stream) = stream_guard.as_mut() {
            stream
                .send(Message::Text(data.to_string()))
                .await
                .map_err(|e| format!("Failed to send WebSocket message: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket connection is closed".to_string())
        }
    }

    /// メッセージを受信
    pub async fn receive(&self) -> Result<Value, String> {
        loop {
            let mut stream_guard = self.stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                match stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let mut result = crate::new_hashmap();
                        result.insert("type".to_string(), Value::String("message".to_string()));
                        result.insert("data".to_string(), Value::String(text));
                        return Ok(Value::Map(result));
                    }
                    Some(Ok(Message::Binary(data))) => {
                        let mut result = crate::new_hashmap();
                        result.insert("type".to_string(), Value::String("binary".to_string()));
                        #[cfg(feature = "string-encoding")]
                        {
                            use base64::{engine::general_purpose, Engine as _};
                            let encoded = general_purpose::STANDARD.encode(&data);
                            result.insert("data".to_string(), Value::String(encoded));
                        }
                        #[cfg(not(feature = "string-encoding"))]
                        {
                            result.insert(
                                "data".to_string(),
                                Value::String(format!("<binary data {} bytes>", data.len())),
                            );
                        }
                        return Ok(Value::Map(result));
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let mut result = crate::new_hashmap();
                        result.insert("type".to_string(), Value::String("close".to_string()));
                        if let Some(CloseFrame { code, reason }) = frame {
                            result
                                .insert("code".to_string(), Value::Integer(u16::from(code) as i64));
                            result.insert("reason".to_string(), Value::String(reason.to_string()));
                        }
                        return Ok(Value::Map(result));
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                        // Ping/Pongは自動的に処理されるのでスキップして次を受信
                        drop(stream_guard);
                        continue;
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Frameは通常ユーザーが直接扱わない
                        return Err("Received unexpected frame".to_string());
                    }
                    Some(Err(e)) => {
                        let mut result = crate::new_hashmap();
                        result.insert("type".to_string(), Value::String("error".to_string()));
                        result.insert("error".to_string(), Value::String(e.to_string()));
                        return Ok(Value::Map(result));
                    }
                    None => {
                        let mut result = crate::new_hashmap();
                        result.insert("type".to_string(), Value::String("close".to_string()));
                        return Ok(Value::Map(result));
                    }
                }
            } else {
                return Err("WebSocket connection is closed".to_string());
            }
        }
    }

    /// 接続をクローズ
    pub async fn close(&self) -> Result<(), String> {
        let mut stream_guard = self.stream.lock().await;
        if let Some(mut stream) = stream_guard.take() {
            stream
                .close(None)
                .await
                .map_err(|e| format!("Failed to close WebSocket connection: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket connection is already closed".to_string())
        }
    }
}

/// グローバルなWebSocket接続マップ
static WS_CONNECTIONS: Lazy<DashMap<i64, Arc<WebSocketConnection>>> = Lazy::new(DashMap::new);

/// 次の接続ID
static NEXT_WS_ID: AtomicI64 = AtomicI64::new(1);

// ========================================
// クライアントサイド関数
// ========================================

/// ws/connect - WebSocketサーバーに接続
///
/// 接続IDを返す（Integer）
pub fn native_ws_connect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["ws/connect", "1"]));
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["ws/connect", "a string"])),
    };

    // 非同期実行
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToCreateRuntime, &[&e.to_string()]))?;

    let url_clone = url.clone();
    let connection = runtime.block_on(async move {
        let (ws_stream, _) = connect_async(&url_clone)
            .await
            .map_err(|e| format!("Failed to connect to WebSocket server: {}", e))?;
        Ok::<_, String>(Arc::new(WebSocketConnection::new(ws_stream)))
    })?;

    // 接続IDを生成してマップに保存
    let conn_id = NEXT_WS_ID.fetch_add(1, Ordering::SeqCst);

    WS_CONNECTIONS.insert(conn_id, connection);

    Ok(Value::Integer(conn_id))
}

/// ws/send - WebSocketでメッセージを送信
pub fn native_ws_send(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["ws/send", "2"]));
    }

    let conn_id = match &args[0] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["ws/send", "a connection ID (Integer)"],
            ))
        }
    };

    let connection = WS_CONNECTIONS
        .get(&conn_id)
        .ok_or_else(|| "WebSocket connection not found".to_string())?;

    let message = match &args[1] {
        Value::String(s) => s.clone(),
        v => format!("{}", v),
    };

    // 非同期実行
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToCreateRuntime, &[&e.to_string()]))?;

    runtime.block_on(async move { connection.send(&message).await })?;

    Ok(Value::Nil)
}

/// ws/receive - WebSocketからメッセージを受信
pub fn native_ws_receive(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["ws/receive", "1"]));
    }

    let conn_id = match &args[0] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["ws/receive", "a connection ID (Integer)"],
            ))
        }
    };

    let connection = WS_CONNECTIONS
        .get(&conn_id)
        .ok_or_else(|| "WebSocket connection not found".to_string())?;

    // 非同期実行
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToCreateRuntime, &[&e.to_string()]))?;

    runtime.block_on(async move { connection.receive().await })
}

/// ws/close - WebSocket接続をクローズ
pub fn native_ws_close(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["ws/close", "1"]));
    }

    let conn_id = match &args[0] {
        Value::Integer(i) => *i,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["ws/close", "a connection ID (Integer)"],
            ))
        }
    };

    let connection = WS_CONNECTIONS
        .get(&conn_id)
        .ok_or_else(|| "WebSocket connection not found".to_string())?;

    // 非同期実行
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| fmt_msg(MsgKey::ServerFailedToCreateRuntime, &[&e.to_string()]))?;

    runtime.block_on(async move { connection.close().await })?;

    // マップから削除
    WS_CONNECTIONS.remove(&conn_id);

    Ok(Value::Nil)
}
