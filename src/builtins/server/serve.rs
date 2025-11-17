//! サーバー起動機能

use super::helpers::{error_response, kw, request_to_value, value_to_response};
use super::routing::route_request;
use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

pub fn native_server_serve(args: &[Value]) -> Result<Value, String> {
    // タイムアウト制限
    const MIN_TIMEOUT_SECS: u64 = 1;
    const MAX_TIMEOUT_SECS: u64 = 300; // 5分
    const DEFAULT_TIMEOUT_SECS: u64 = 30;

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

    let port_key = kw("port");
    let host_key = kw("host");
    let timeout_key = kw("timeout");

    let port = match opts.get(&port_key) {
        Some(Value::Integer(p)) => *p as u16,
        _ => 3000,
    };

    let host = match opts.get(&host_key) {
        Some(Value::String(h)) => h.clone(),
        _ => "127.0.0.1".to_string(),
    };

    let timeout_secs = match opts.get(&timeout_key) {
        Some(Value::Integer(t))
            if *t >= MIN_TIMEOUT_SECS as i64 && *t <= MAX_TIMEOUT_SECS as i64 =>
        {
            *t as u64
        }
        Some(Value::Integer(t)) if *t > MAX_TIMEOUT_SECS as i64 => {
            eprintln!(
                "Warning: Timeout {} exceeds maximum {}. Using maximum.",
                t, MAX_TIMEOUT_SECS
            );
            MAX_TIMEOUT_SECS
        }
        Some(Value::Integer(t)) if *t < MIN_TIMEOUT_SECS as i64 => {
            eprintln!(
                "Warning: Timeout {} is below minimum {}. Using default.",
                t, MIN_TIMEOUT_SECS
            );
            DEFAULT_TIMEOUT_SECS
        }
        _ => DEFAULT_TIMEOUT_SECS,
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
