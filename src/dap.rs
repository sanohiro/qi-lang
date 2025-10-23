//! Debug Adapter Protocol (DAP) サーバー実装
//!
//! VSCodeとの統合のためのDAPサーバー

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DAPプロトコルのメッセージベース
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub seq: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
}

/// リクエストメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub seq: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// レスポンスメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub seq: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub request_seq: i64,
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// イベントメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub seq: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// Output イベントのbody
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEventBody {
    pub category: String,
    pub output: String,
}

// ========================================
// Initialize関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    pub adapter_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default)]
    pub lines_start_at1: bool,
    #[serde(default)]
    pub columns_start_at1: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_configuration_done_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_function_breakpoints: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_conditional_breakpoints: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_evaluate_for_hovers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_back: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_set_variable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_restart_frame: Option<bool>,
}

// ========================================
// Launch/Attach関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_on_entry: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRequestArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
}

// ========================================
// Breakpoint関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArguments {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    pub line: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub id: i64,
    pub verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

// ========================================
// Thread/StackTrace関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceArguments {
    pub thread_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    pub line: i64,
    pub column: i64,
}

// ========================================
// Scope/Variable関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesArguments {
    pub frame_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub name: String,
    pub variables_reference: i64,
    pub expensive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesArguments {
    pub variables_reference: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub var_type: Option<String>,
    pub variables_reference: i64,
}

// ========================================
// Continue/Step関連
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueArguments {
    pub thread_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextArguments {
    pub thread_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInArguments {
    pub thread_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepOutArguments {
    pub thread_id: i64,
}

// ========================================
// DAPサーバー
// ========================================

pub struct DapServer {
    seq: std::sync::atomic::AtomicI64,
    initialized: std::sync::atomic::AtomicBool,
    breakpoints: parking_lot::RwLock<HashMap<String, Vec<SourceBreakpoint>>>,
}

impl DapServer {
    pub fn new() -> Self {
        DapServer {
            seq: std::sync::atomic::AtomicI64::new(1),
            initialized: std::sync::atomic::AtomicBool::new(false),
            breakpoints: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    fn next_seq(&self) -> i64 {
        self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn handle_request(&self, request: Request) -> Response {
        match request.command.as_str() {
            "initialize" => self.handle_initialize(request),
            "launch" => self.handle_launch(request),
            "attach" => self.handle_attach(request),
            "setBreakpoints" => self.handle_set_breakpoints(request),
            "configurationDone" => self.handle_configuration_done(request),
            "threads" => self.handle_threads(request),
            "stackTrace" => self.handle_stack_trace(request),
            "scopes" => self.handle_scopes(request),
            "variables" => self.handle_variables(request),
            "continue" => self.handle_continue(request),
            "next" => self.handle_next(request),
            "stepIn" => self.handle_step_in(request),
            "stepOut" => self.handle_step_out(request),
            "disconnect" => self.handle_disconnect(request),
            _ => Response {
                seq: self.next_seq(),
                msg_type: "response".to_string(),
                request_seq: request.seq,
                success: false,
                command: request.command.clone(),
                message: Some(format!("Unknown command: {}", request.command)),
                body: None,
            },
        }
    }

    /// 非同期版リクエストハンドラ（イベント送信チャネル付き）
    pub fn handle_request_async(
        &self,
        request: Request,
        event_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Response {
        match request.command.as_str() {
            "launch" => self.handle_launch_async(request, event_tx),
            _ => {
                // 他のコマンドは同期版を使用
                self.handle_request(request)
            }
        }
    }

    /// 非同期版launchハンドラ（プログラムを実行してイベント送信）
    fn handle_launch_async(
        &self,
        request: Request,
        event_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Response {
        // launchの引数を解析
        if let Some(args) = request.arguments {
            if let Ok(launch_args) = serde_json::from_value::<LaunchRequestArguments>(args) {
                if let Some(program) = launch_args.program {
                    // プログラムをバックグラウンドで実行
                    let seq_base = self.next_seq();
                    let program_clone = program.clone();
                    tokio::spawn(async move {
                        // プログラム開始メッセージ
                        let start_msg = OutputEventBody {
                            category: "console".to_string(),
                            output: format!("Starting program: {}\n", program_clone),
                        };
                        let start_event = Event {
                            seq: seq_base,
                            msg_type: "event".to_string(),
                            event: "output".to_string(),
                            body: serde_json::to_value(&start_msg).ok(),
                        };
                        if let Ok(event_json) = serde_json::to_string(&start_event) {
                            let _ = event_tx.send(event_json).await;
                        }

                        // プログラム実行
                        match run_qi_program_async(&program_clone, event_tx.clone(), seq_base).await
                        {
                            Ok(_) => {
                                // 成功メッセージ
                                let success_msg = OutputEventBody {
                                    category: "console".to_string(),
                                    output: "\nProgram completed successfully.\n".to_string(),
                                };
                                let success_event = Event {
                                    seq: seq_base + 1,
                                    msg_type: "event".to_string(),
                                    event: "output".to_string(),
                                    body: serde_json::to_value(&success_msg).ok(),
                                };
                                if let Ok(event_json) = serde_json::to_string(&success_event) {
                                    let _ = event_tx.send(event_json).await;
                                }
                            }
                            Err(e) => {
                                // エラーメッセージ
                                let error_msg = OutputEventBody {
                                    category: "stderr".to_string(),
                                    output: format!("\nProgram failed: {}\n", e),
                                };
                                let error_event = Event {
                                    seq: seq_base + 1,
                                    msg_type: "event".to_string(),
                                    event: "output".to_string(),
                                    body: serde_json::to_value(&error_msg).ok(),
                                };
                                if let Ok(event_json) = serde_json::to_string(&error_event) {
                                    let _ = event_tx.send(event_json).await;
                                }
                            }
                        }

                        // terminatedイベント送信
                        let terminated_event = Event {
                            seq: seq_base + 2,
                            msg_type: "event".to_string(),
                            event: "terminated".to_string(),
                            body: None,
                        };

                        if let Ok(event_json) = serde_json::to_string(&terminated_event) {
                            let _ = event_tx.send(event_json).await;
                        }
                    });

                    return Response {
                        seq: self.next_seq(),
                        msg_type: "response".to_string(),
                        request_seq: request.seq,
                        success: true,
                        command: "launch".to_string(),
                        message: None,
                        body: None,
                    };
                } else {
                    return Response {
                        seq: self.next_seq(),
                        msg_type: "response".to_string(),
                        request_seq: request.seq,
                        success: false,
                        command: "launch".to_string(),
                        message: Some("No program specified".to_string()),
                        body: None,
                    };
                }
            }
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: false,
            command: "launch".to_string(),
            message: Some("Invalid launch arguments".to_string()),
            body: None,
        }
    }

    fn handle_launch(&self, request: Request) -> Response {
        // launchの引数を解析
        if let Some(args) = request.arguments {
            if let Ok(launch_args) = serde_json::from_value::<LaunchRequestArguments>(args) {
                if let Some(_program) = launch_args.program {
                    // NOTE: この同期版handle_launchは非推奨。run_async()を使用すること
                    // プログラム実行は非同期版（handle_launch_async）で実装済み
                } else {
                    return Response {
                        seq: self.next_seq(),
                        msg_type: "response".to_string(),
                        request_seq: request.seq,
                        success: false,
                        command: "launch".to_string(),
                        message: Some("No program specified".to_string()),
                        body: None,
                    };
                }
            }
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "launch".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_attach(&self, request: Request) -> Response {
        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "attach".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_initialize(&self, request: Request) -> Response {
        let capabilities = Capabilities {
            supports_configuration_done_request: Some(true),
            supports_function_breakpoints: Some(false),
            supports_conditional_breakpoints: Some(true),
            supports_evaluate_for_hovers: Some(false),
            supports_step_back: Some(false),
            supports_set_variable: Some(false),
            supports_restart_frame: Some(false),
        };

        self.initialized
            .store(true, std::sync::atomic::Ordering::SeqCst);

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "initialize".to_string(),
            message: None,
            body: Some(serde_json::to_value(capabilities).unwrap()),
        }
    }

    fn handle_set_breakpoints(&self, request: Request) -> Response {
        if let Some(args) = request.arguments {
            if let Ok(args) = serde_json::from_value::<SetBreakpointsArguments>(args) {
                let path = args.source.path.clone().unwrap_or_default();
                let breakpoints = args.breakpoints.unwrap_or_default();

                // ブレークポイントを保存
                self.breakpoints
                    .write()
                    .insert(path.clone(), breakpoints.clone());

                // グローバルデバッガにブレークポイントを設定
                if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
                    dbg.clear_breakpoints();
                    for bp in &breakpoints {
                        dbg.add_breakpoint(&path, bp.line as usize);
                    }
                }

                // レスポンスのブレークポイントを生成
                let response_bps: Vec<Breakpoint> = breakpoints
                    .iter()
                    .enumerate()
                    .map(|(i, bp)| Breakpoint {
                        id: i as i64,
                        verified: true,
                        line: Some(bp.line),
                        source: Some(args.source.clone()),
                    })
                    .collect();

                let body = serde_json::json!({ "breakpoints": response_bps });

                return Response {
                    seq: self.next_seq(),
                    msg_type: "response".to_string(),
                    request_seq: request.seq,
                    success: true,
                    command: "setBreakpoints".to_string(),
                    message: None,
                    body: Some(body),
                };
            }
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: false,
            command: "setBreakpoints".to_string(),
            message: Some("Invalid arguments".to_string()),
            body: None,
        }
    }

    fn handle_threads(&self, request: Request) -> Response {
        let threads = vec![Thread {
            id: 1,
            name: "Main Thread".to_string(),
        }];

        let body = serde_json::json!({ "threads": threads });

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "threads".to_string(),
            message: None,
            body: Some(body),
        }
    }

    fn handle_stack_trace(&self, request: Request) -> Response {
        // デバッガからスタック情報を取得
        let stack_frames = if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
            dbg.call_stack()
                .iter()
                .enumerate()
                .map(|(i, frame)| StackFrame {
                    id: i as i64,
                    name: frame.function_name.clone(),
                    source: Some(Source {
                        name: None,
                        path: Some(frame.file.clone()),
                    }),
                    line: frame.line as i64,
                    column: frame.column as i64,
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let body = serde_json::json!({
            "stackFrames": stack_frames,
            "totalFrames": stack_frames.len()
        });

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stackTrace".to_string(),
            message: None,
            body: Some(body),
        }
    }

    fn handle_scopes(&self, _request: Request) -> Response {
        // TODO: 実際のスコープ情報を取得
        let scopes = vec![Scope {
            name: "Local".to_string(),
            variables_reference: 1,
            expensive: false,
        }];

        let body = serde_json::json!({ "scopes": scopes });

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: _request.seq,
            success: true,
            command: "scopes".to_string(),
            message: None,
            body: Some(body),
        }
    }

    fn handle_variables(&self, _request: Request) -> Response {
        // TODO: 実際の変数情報を取得
        let variables: Vec<Variable> = vec![];

        let body = serde_json::json!({ "variables": variables });

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: _request.seq,
            success: true,
            command: "variables".to_string(),
            message: None,
            body: Some(body),
        }
    }

    fn handle_continue(&self, request: Request) -> Response {
        // デバッガを再開
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.resume();
        }

        let body = serde_json::json!({ "allThreadsContinued": true });

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "continue".to_string(),
            message: None,
            body: Some(body),
        }
    }

    fn handle_next(&self, request: Request) -> Response {
        // ステップオーバー
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_over();
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "next".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_step_in(&self, request: Request) -> Response {
        // ステップイン
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_in();
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stepIn".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_step_out(&self, request: Request) -> Response {
        // ステップアウト
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_out();
        }

        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stepOut".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_configuration_done(&self, request: Request) -> Response {
        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "configurationDone".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_disconnect(&self, request: Request) -> Response {
        Response {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "disconnect".to_string(),
            message: None,
            body: None,
        }
    }

    pub fn create_stopped_event(&self, reason: &str, thread_id: i64) -> Event {
        let body = serde_json::json!({
            "reason": reason,
            "threadId": thread_id,
            "allThreadsStopped": true
        });

        Event {
            seq: self.next_seq(),
            msg_type: "event".to_string(),
            event: "stopped".to_string(),
            body: Some(body),
        }
    }

    pub fn create_initialized_event(&self) -> Event {
        Event {
            seq: self.next_seq(),
            msg_type: "event".to_string(),
            event: "initialized".to_string(),
            body: None,
        }
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// DAP通信レイヤー（Content-Length形式）
// ========================================

use std::io::{self, BufRead, BufReader, Write};

#[cfg(feature = "dap-server")]
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

/// DAPメッセージを読み取る（Content-Length形式）
///
/// # フォーマット
/// ```
/// Content-Length: 119\r\n
/// \r\n
/// {"seq":1,"type":"request","command":"initialize",...}
/// ```
pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<String> {
    // Content-Lengthヘッダーを読む
    let mut header = String::new();
    let bytes_read = reader.read_line(&mut header)?;

    // EOFチェック
    if bytes_read == 0 || header.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Connection closed",
        ));
    }

    if !header.starts_with("Content-Length: ") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid header: {}", header),
        ));
    }

    let length: usize = header
        .trim_start_matches("Content-Length: ")
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // 空行を読む（\r\n）
    let mut empty_line = String::new();
    reader.read_line(&mut empty_line)?;

    // メッセージ本体を読む
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;

    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// DAPメッセージを書き出す（Content-Length形式）
pub fn write_message<W: Write>(writer: &mut W, message: &str) -> io::Result<()> {
    let content_length = message.len();
    write!(
        writer,
        "Content-Length: {}\r\n\r\n{}",
        content_length, message
    )?;
    writer.flush()?;
    Ok(())
}

// ========================================
// 非同期版DAP通信レイヤー
// ========================================

/// DAPメッセージを非同期で読み取る（Content-Length形式）
pub async fn read_message_async<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> io::Result<String> {
    // Content-Lengthヘッダーを読む
    let mut header = String::new();
    let bytes_read = reader.read_line(&mut header).await?;

    // EOFチェック
    if bytes_read == 0 || header.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Connection closed",
        ));
    }

    if !header.starts_with("Content-Length: ") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid header: {}", header),
        ));
    }

    let length: usize = header
        .trim_start_matches("Content-Length: ")
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // 空行を読む（\r\n）
    let mut empty_line = String::new();
    reader.read_line(&mut empty_line).await?;

    // メッセージ本体を読む
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer).await?;

    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// DAPメッセージを非同期で書き出す（Content-Length形式）
pub async fn write_message_async<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    message: &str,
) -> io::Result<()> {
    let content_length = message.len();
    let header = format!("Content-Length: {}\r\n\r\n{}", content_length, message);
    writer.write_all(header.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

// ========================================
// DAPサーバーメインループ
// ========================================

impl DapServer {
    /// DAPサーバーを起動（非同期版・推奨）
    pub async fn run_async_internal() -> io::Result<()> {
        let server = std::sync::Arc::new(DapServer::new());
        let stdin = tokio::io::stdin();

        // オリジナルのstdoutハンドルを保存（プログラム実行時のリダイレクトの影響を受けない）
        #[cfg(unix)]
        let original_stdout_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
        #[cfg(unix)]
        if original_stdout_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        #[cfg(windows)]
        let original_stdout_handle = unsafe {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::System::Console::*;
            GetStdHandle(STD_OUTPUT_HANDLE)
        };

        // オリジナルのstdoutからWriterを作成
        #[cfg(unix)]
        let stdout_file = unsafe {
            use std::os::unix::io::FromRawFd;
            std::fs::File::from_raw_fd(original_stdout_fd)
        };

        #[cfg(windows)]
        let stdout_file = unsafe {
            use std::os::windows::io::FromRawHandle;
            std::fs::File::from_raw_handle(original_stdout_handle as _)
        };

        let mut writer = tokio::io::BufWriter::new(tokio::fs::File::from_std(stdout_file));
        let mut reader = AsyncBufReader::new(stdin);

        // イベント送信用のchannel
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<String>(100);

        // メインループ
        loop {
            tokio::select! {
                // リクエスト処理
                message_result = read_message_async(&mut reader) => {
                    match message_result {
                        Ok(message) => {
                            // JSONパース
                            let request: Request = match serde_json::from_str(&message) {
                                Ok(req) => req,
                                Err(_) => {
                                    continue;
                                }
                            };

                            // リクエスト処理（イベント送信チャネルを渡す）
                            let response = server.handle_request_async(request, event_tx.clone());

                            // レスポンス送信
                            let response_json = serde_json::to_string(&response)
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                            write_message_async(&mut writer, &response_json).await?;

                            // initialized イベント送信（initializeリクエスト後）
                            if response.command == "initialize" && response.success {
                                let initialized_event = server.create_initialized_event();
                                let event_json = serde_json::to_string(&initialized_event)
                                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                                write_message_async(&mut writer, &event_json).await?;
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                // イベント送信
                Some(event_json) = event_rx.recv() => {
                    write_message_async(&mut writer, &event_json).await?;
                }
            }
        }

        Ok(())
    }

    /// DAPサーバーを起動（エントリーポイント）
    #[tokio::main]
    pub async fn run_async() -> io::Result<()> {
        Self::run_async_internal().await
    }

    /// DAPサーバーを起動（同期版・互換性のため残す）
    pub fn run() -> io::Result<()> {
        let server = DapServer::new();
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        loop {
            // リクエストを読み取る
            let message = match read_message(&mut reader) {
                Ok(msg) => msg,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(_) => {
                    continue;
                }
            };

            // JSONパース
            let request: Request = match serde_json::from_str(&message) {
                Ok(req) => req,
                Err(_) => {
                    continue;
                }
            };

            // リクエスト処理
            let response = server.handle_request(request);

            // レスポンス送信
            let response_json = serde_json::to_string(&response)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            write_message(&mut stdout, &response_json)?;

            // initialized イベント送信（initializeリクエスト後）
            if response.command == "initialize" && response.success {
                let initialized_event = server.create_initialized_event();
                let event_json = serde_json::to_string(&initialized_event)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                write_message(&mut stdout, &event_json)?;
            }
        }

        Ok(())
    }
}

// ========================================
// Qiプログラム実行
// ========================================

/// Qiプログラムを非同期実行（stdoutリダイレクト使用）
async fn run_qi_program_async(
    program_path: &str,
    event_tx: tokio::sync::mpsc::Sender<String>,
    seq_base: i64,
) -> Result<(), String> {
    use crate::eval::Evaluator;
    use crate::parser::Parser;

    // ファイル読み込み
    let content = tokio::fs::read_to_string(program_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let program_path_owned = program_path.to_string();

    // stdout/stderrをリダイレクト
    let redirect = stdio_redirect::StdioRedirect::new()
        .map_err(|e| format!("Failed to redirect stdio: {}", e))?;

    // パイプから読み取るタスクを起動（stdout用とstderr用）
    let stdout_handle = redirect.spawn_stdout_reader(event_tx.clone(), seq_base);
    let stderr_handle = redirect.spawn_stderr_reader(event_tx.clone(), seq_base);

    // Evaluatorの実行は同期的なのでspawn_blockingで実行
    let result = tokio::task::spawn_blocking(move || {
        let evaluator = Evaluator::new();
        evaluator.set_source(program_path_owned.clone(), content.clone());

        // パース
        let mut parser = Parser::new(&content).map_err(|e| format!("Parser error: {}", e))?;
        parser.set_source_name(program_path_owned.clone());
        let exprs = parser
            .parse_all()
            .map_err(|e| format!("Parse error: {}", e))?;

        // 評価
        for expr in exprs.iter() {
            evaluator
                .eval(expr)
                .map_err(|e| format!("Runtime error: {}", e))?;
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    // stdio を元に戻す
    drop(redirect);

    // 読み取りタスクの完了を待つ
    let _ = tokio::join!(stdout_handle, stderr_handle);

    result
}

// ========================================
// stdio リダイレクト（クロスプラットフォーム）
// ========================================

mod stdio_redirect {
    use super::{Event, OutputEventBody};
    use std::io::{self, Read};

    #[cfg(unix)]
    pub struct StdioRedirect {
        original_stdout: i32,
        original_stderr: i32,
        stdout_read_fd: i32,
        stderr_read_fd: i32,
    }

    #[cfg(windows)]
    pub struct StdioRedirect {
        original_stdout: windows_sys::Win32::Foundation::HANDLE,
        original_stderr: windows_sys::Win32::Foundation::HANDLE,
        stdout_read_handle: windows_sys::Win32::Foundation::HANDLE,
        stderr_read_handle: windows_sys::Win32::Foundation::HANDLE,
    }

    impl StdioRedirect {
        /// stdout/stderrをパイプにリダイレクト
        #[cfg(unix)]
        pub fn new() -> io::Result<Self> {
            unsafe {
                // 元のstdoutを保存
                let original_stdout = libc::dup(libc::STDOUT_FILENO);
                if original_stdout < 0 {
                    return Err(io::Error::last_os_error());
                }

                // 元のstderrを保存
                let original_stderr = libc::dup(libc::STDERR_FILENO);
                if original_stderr < 0 {
                    libc::close(original_stdout);
                    return Err(io::Error::last_os_error());
                }

                // stdout用パイプ作成
                let mut stdout_pipe: [i32; 2] = [0, 0];
                if libc::pipe(stdout_pipe.as_mut_ptr()) < 0 {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    return Err(io::Error::last_os_error());
                }

                // stderr用パイプ作成
                let mut stderr_pipe: [i32; 2] = [0, 0];
                if libc::pipe(stderr_pipe.as_mut_ptr()) < 0 {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(stdout_pipe[0]);
                    libc::close(stdout_pipe[1]);
                    return Err(io::Error::last_os_error());
                }

                // stdoutをパイプの書き込み側にリダイレクト
                if libc::dup2(stdout_pipe[1], libc::STDOUT_FILENO) < 0 {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(stdout_pipe[0]);
                    libc::close(stdout_pipe[1]);
                    libc::close(stderr_pipe[0]);
                    libc::close(stderr_pipe[1]);
                    return Err(io::Error::last_os_error());
                }

                // stderrをパイプの書き込み側にリダイレクト
                if libc::dup2(stderr_pipe[1], libc::STDERR_FILENO) < 0 {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(stdout_pipe[0]);
                    libc::close(stdout_pipe[1]);
                    libc::close(stderr_pipe[0]);
                    libc::close(stderr_pipe[1]);
                    return Err(io::Error::last_os_error());
                }

                // 書き込み側は不要（stdout/stderrが参照している）
                libc::close(stdout_pipe[1]);
                libc::close(stderr_pipe[1]);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read_fd: stdout_pipe[0],
                    stderr_read_fd: stderr_pipe[0],
                })
            }
        }

        /// stdout/stderrをパイプにリダイレクト（Windows版）
        #[cfg(windows)]
        pub fn new() -> io::Result<Self> {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::Storage::FileSystem::*;
            use windows_sys::Win32::System::Console::*;
            use windows_sys::Win32::System::Pipes::*;

            unsafe {
                // 元のstdoutを保存
                let original_stdout = GetStdHandle(STD_OUTPUT_HANDLE);
                if original_stdout == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }

                // 元のstderrを保存
                let original_stderr = GetStdHandle(STD_ERROR_HANDLE);
                if original_stderr == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }

                // stdout用パイプ作成
                let mut stdout_read_handle: HANDLE = 0;
                let mut stdout_write_handle: HANDLE = 0;
                if CreatePipe(
                    &mut stdout_read_handle,
                    &mut stdout_write_handle,
                    std::ptr::null(),
                    0,
                ) == 0
                {
                    return Err(io::Error::last_os_error());
                }

                // stderr用パイプ作成
                let mut stderr_read_handle: HANDLE = 0;
                let mut stderr_write_handle: HANDLE = 0;
                if CreatePipe(
                    &mut stderr_read_handle,
                    &mut stderr_write_handle,
                    std::ptr::null(),
                    0,
                ) == 0
                {
                    CloseHandle(stdout_read_handle);
                    CloseHandle(stdout_write_handle);
                    return Err(io::Error::last_os_error());
                }

                // stdoutをパイプの書き込み側に設定
                if SetStdHandle(STD_OUTPUT_HANDLE, stdout_write_handle) == 0 {
                    CloseHandle(stdout_read_handle);
                    CloseHandle(stdout_write_handle);
                    CloseHandle(stderr_read_handle);
                    CloseHandle(stderr_write_handle);
                    return Err(io::Error::last_os_error());
                }

                // stderrをパイプの書き込み側に設定
                if SetStdHandle(STD_ERROR_HANDLE, stderr_write_handle) == 0 {
                    CloseHandle(stdout_read_handle);
                    CloseHandle(stdout_write_handle);
                    CloseHandle(stderr_read_handle);
                    CloseHandle(stderr_write_handle);
                    return Err(io::Error::last_os_error());
                }

                // 書き込み側ハンドルを閉じる（stdout/stderrが参照している）
                CloseHandle(stdout_write_handle);
                CloseHandle(stderr_write_handle);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read_handle,
                    stderr_read_handle,
                })
            }
        }

        /// stdoutパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stdout_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(
                event_tx,
                seq_base,
                "stdout",
                #[cfg(unix)]
                self.stdout_read_fd,
                #[cfg(windows)]
                self.stdout_read_handle,
            )
        }

        /// stderrパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stderr_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(
                event_tx,
                seq_base,
                "stderr",
                #[cfg(unix)]
                self.stderr_read_fd,
                #[cfg(windows)]
                self.stderr_read_handle,
            )
        }

        /// パイプから読み取ってDAPイベントを送信する共通実装
        fn spawn_reader_impl(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
            category: &'static str,
            #[cfg(unix)] read_fd: i32,
            #[cfg(windows)] read_handle: windows_sys::Win32::Foundation::HANDLE,
        ) -> tokio::task::JoinHandle<()> {
            #[cfg(unix)]
            let read_fd_dup = unsafe { libc::dup(read_fd) };

            tokio::spawn(async move {
                #[cfg(unix)]
                let mut reader = unsafe {
                    use std::os::unix::io::FromRawFd;
                    std::fs::File::from_raw_fd(read_fd_dup)
                };

                #[cfg(windows)]
                let mut reader = unsafe {
                    use std::os::windows::io::FromRawHandle;
                    use windows_sys::Win32::Foundation::DuplicateHandle;
                    use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
                    use windows_sys::Win32::System::Threading::GetCurrentProcess;

                    let mut dup_handle: windows_sys::Win32::Foundation::HANDLE = 0;
                    if DuplicateHandle(
                        GetCurrentProcess(),
                        read_handle,
                        GetCurrentProcess(),
                        &mut dup_handle,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    ) == 0
                    {
                        // Failed to duplicate handle - return silently
                        return;
                    }
                    std::fs::File::from_raw_handle(dup_handle as _)
                };

                let mut buffer = [0u8; 4096];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            if let Ok(output) = String::from_utf8(buffer[..n].to_vec()) {
                                let output_msg = OutputEventBody {
                                    category: category.to_string(),
                                    output,
                                };
                                let output_event = Event {
                                    seq: seq_base,
                                    msg_type: "event".to_string(),
                                    event: "output".to_string(),
                                    body: serde_json::to_value(&output_msg).ok(),
                                };
                                if let Ok(event_json) = serde_json::to_string(&output_event) {
                                    let _ = event_tx.send(event_json).await;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
        }
    }

    impl Drop for StdioRedirect {
        /// stdout/stderrを元に戻す
        #[cfg(unix)]
        fn drop(&mut self) {
            unsafe {
                libc::dup2(self.original_stdout, libc::STDOUT_FILENO);
                libc::dup2(self.original_stderr, libc::STDERR_FILENO);
                libc::close(self.original_stdout);
                libc::close(self.original_stderr);
                libc::close(self.stdout_read_fd);
                libc::close(self.stderr_read_fd);
            }
        }

        #[cfg(windows)]
        fn drop(&mut self) {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::System::Console::*;

            unsafe {
                SetStdHandle(STD_OUTPUT_HANDLE, self.original_stdout);
                SetStdHandle(STD_ERROR_HANDLE, self.original_stderr);
                CloseHandle(self.stdout_read_handle);
                CloseHandle(self.stderr_read_handle);
            }
        }
    }
}
