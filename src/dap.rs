//! Debug Adapter Protocol (DAP) サーバー実装
//!
//! VSCodeとの統合のためのDAPサーバー

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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

/// launch時の保留情報
struct PendingLaunch {
    program: String,
    event_tx: tokio::sync::mpsc::Sender<String>,
}

pub struct DapServer {
    seq: std::sync::atomic::AtomicI64,
    initialized: std::sync::atomic::AtomicBool,
    breakpoints: parking_lot::RwLock<HashMap<String, Vec<SourceBreakpoint>>>,
    pending_launch: parking_lot::RwLock<Option<PendingLaunch>>,
}

impl DapServer {
    pub fn new() -> Self {
        DapServer {
            seq: std::sync::atomic::AtomicI64::new(1),
            initialized: std::sync::atomic::AtomicBool::new(false),
            breakpoints: parking_lot::RwLock::new(HashMap::new()),
            pending_launch: parking_lot::RwLock::new(None),
        }
    }

    fn next_seq(&self) -> i64 {
        self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// イベントを送信
    pub fn send_event(&self, event_name: &str, body: Option<serde_json::Value>) -> Event {
        Event {
            seq: self.next_seq(),
            msg_type: "event".to_string(),
            event: event_name.to_string(),
            body,
        }
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
            "writeStdin" => self.handle_write_stdin(request),
            "evaluate" => self.handle_evaluate(request),
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
            "configurationDone" => self.handle_configuration_done_async(request, event_tx),
            _ => {
                // 他のコマンドは同期版を使用
                self.handle_request(request)
            }
        }
    }

    /// 非同期版launchハンドラ（configurationDone後に実行するため情報を保存）
    fn handle_launch_async(
        &self,
        request: Request,
        event_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Response {
        // launchの引数を解析
        if let Some(args) = request.arguments {
            if let Ok(launch_args) = serde_json::from_value::<LaunchRequestArguments>(args) {
                if let Some(program) = launch_args.program {
                    let log_msg = format!(
                        "[DAP] Launch request received: program={} (pending until configurationDone)\n",
                        program
                    );
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

                    // プログラム実行情報を保存（configurationDone後に実行）
                    *self.pending_launch.write() = Some(PendingLaunch {
                        program: program.clone(),
                        event_tx,
                    });

                    let log_msg = format!("[DAP] Stored pending launch for: {}\n", program);
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

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
        let log_msg = format!("setBreakpoints called\n");
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();

        if let Some(args) = request.arguments {
            if let Ok(args) = serde_json::from_value::<SetBreakpointsArguments>(args) {
                let path = args.source.path.clone().unwrap_or_default();
                let breakpoints = args.breakpoints.unwrap_or_default();

                let log_msg = format!(
                    "  path: {}\n  breakpoints: {} items\n",
                    path,
                    breakpoints.len()
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

                // ブレークポイントを保存
                self.breakpoints
                    .write()
                    .insert(path.clone(), breakpoints.clone());

                // グローバルデバッガにブレークポイントを設定
                if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
                    // このファイルのブレークポイントだけをクリア（他のファイルは残す）
                    let log_msg = format!("  Clearing breakpoints for file: {}\n", path);
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();
                    dbg.clear_breakpoints_for_file(&path);
                    for bp in &breakpoints {
                        let log_msg = format!("  Setting BP: {}:{}\n", path, bp.line);
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();
                        dap_log(&format!("[DAP] Setting breakpoint: {}:{}", path, bp.line));
                        dbg.add_breakpoint(&path, bp.line as usize);
                    }
                } else {
                    let log_msg = "  ERROR: GLOBAL_DEBUGGER is None!\n";
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();
                    dap_log("[DAP] WARNING: GLOBAL_DEBUGGER is not initialized!");
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
        let mut stack_frames = if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
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

        // call_stackが空の場合（トップレベルコード）、stopped_eventから現在位置を取得
        if stack_frames.is_empty() {
            if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
                if let Some((file, line, column, _reason)) = dbg.get_stopped_event() {
                    stack_frames.push(StackFrame {
                        id: 0,
                        name: "<main>".to_string(),
                        source: Some(Source {
                            name: None,
                            path: Some(file.clone()),
                        }),
                        line: *line as i64,
                        column: *column as i64,
                    });
                }
            }
        }

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
        // 停止時の環境から変数一覧を取得
        let mut variables: Vec<Variable> = vec![];

        if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
            if let Some(env_arc) = dbg.get_stopped_env() {
                let env = env_arc.read();

                // ローカル変数のみを取得（親環境にあるグローバル変数/関数を除外）
                for (name, binding) in env.local_bindings() {
                    // 関数・マクロ・ネイティブ関数は除外（データのみ表示）
                    use crate::value::Value;
                    let is_callable = matches!(
                        binding.value,
                        Value::Function(_) | Value::Macro(_) | Value::NativeFunc(_)
                    );

                    if !is_callable {
                        variables.push(Variable {
                            name,
                            value: format!("{}", binding.value),
                            var_type: Some(binding.value.type_name().to_string()),
                            variables_reference: 0, // TODO: ネストした値の展開は後で実装
                        });
                    }
                }
            }
        }

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
            dbg.resume(); // 待機中のスレッドを起こす
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
            dbg.resume(); // 待機中のスレッドを起こす
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
            dbg.resume(); // 待機中のスレッドを起こす
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

    /// 非同期版configurationDoneハンドラ（pending_launchがあれば実行開始）
    fn handle_configuration_done_async(
        &self,
        request: Request,
        _event_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Response {
        let log_msg = "[DAP] configurationDone received - checking for pending launch\n";
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();

        // pending_launchを取り出す
        let pending = self.pending_launch.write().take();

        if let Some(PendingLaunch { program, event_tx }) = pending {
            let log_msg = format!("[DAP] Found pending launch: {}\n", program);
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/qi-dap.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                .ok();

            // プログラムをバックグラウンドで実行
            let seq_base = self.next_seq();
            let program_clone = program.clone();

            let log_msg = format!("[DAP] Spawning task to run program: {}\n", program_clone);
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/qi-dap.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                .ok();

            tokio::spawn(async move {
                let log_msg = format!(
                    "[DAP] Task started - launching program: {}\n",
                    program_clone
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

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
                let log_msg = format!(
                    "[DAP] Calling run_qi_program_async for: {}\n",
                    program_clone
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

                match run_qi_program_async(&program_clone, event_tx.clone(), seq_base).await {
                    Ok(_) => {
                        let log_msg = "[DAP] Program completed successfully\n";
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();

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
                        let log_msg = format!("[DAP] Program failed: {}\n", e);
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();

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
                let log_msg = "[DAP] Sending terminated event\n";
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

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
        } else {
            let log_msg = "[DAP] No pending launch found\n";
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/qi-dap.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                .ok();
        }

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

    /// evaluateリクエストハンドラー（デバッグコンソール入力）
    ///
    /// デバッグコンソールから入力された式を評価します。
    /// `.stdin <text>` 形式の入力を標準入力に書き込みます。
    /// 引数: { "expression": "式またはコマンド", "frameId": ..., "context": ... }
    fn handle_evaluate(&self, request: Request) -> Response {
        let expression = request
            .arguments
            .as_ref()
            .and_then(|args| args.get("expression"))
            .and_then(|v| v.as_str());

        match expression {
            Some(expr) => {
                // .stdin プレフィックスをチェック
                if let Some(text) = expr.strip_prefix(".stdin ") {
                    // エスケープシーケンスを変換（\n → 改行, \t → タブ）
                    let text = text.replace("\\n", "\n").replace("\\t", "\t");
                    // writeStdinを実行
                    match write_to_stdin(&text) {
                        Ok(()) => Response {
                            seq: self.next_seq(),
                            msg_type: "response".to_string(),
                            request_seq: request.seq,
                            success: true,
                            command: "evaluate".to_string(),
                            message: None,
                            body: Some(serde_json::json!({
                                "result": crate::i18n::fmt_ui_msg(crate::i18n::UiMsg::DapStdinSent, &[&text]),
                                "variablesReference": 0
                            })),
                        },
                        Err(e) => Response {
                            seq: self.next_seq(),
                            msg_type: "response".to_string(),
                            request_seq: request.seq,
                            success: false,
                            command: "evaluate".to_string(),
                            message: Some(format!("Failed to write to stdin: {}", e)),
                            body: None,
                        },
                    }
                } else {
                    // ウォッチ式の評価
                    if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
                        if let Some(env_arc) = dbg.get_stopped_env() {
                            // パーサーと評価器を使って式を評価
                            match crate::parser::Parser::new(expr).and_then(|mut p| p.parse_all()) {
                                Ok(exprs) => {
                                    if exprs.is_empty() {
                                        return Response {
                                            seq: self.next_seq(),
                                            msg_type: "response".to_string(),
                                            request_seq: request.seq,
                                            success: false,
                                            command: "evaluate".to_string(),
                                            message: Some(crate::i18n::fmt_msg(crate::i18n::MsgKey::DapEmptyExpression, &[])),
                                            body: None,
                                        };
                                    }

                                    // 最初の式を評価
                                    let evaluator = crate::eval::Evaluator::new();
                                    match evaluator.eval_with_env(&exprs[0], env_arc.clone()) {
                                        Ok(value) => Response {
                                            seq: self.next_seq(),
                                            msg_type: "response".to_string(),
                                            request_seq: request.seq,
                                            success: true,
                                            command: "evaluate".to_string(),
                                            message: None,
                                            body: Some(serde_json::json!({
                                                "result": format!("{}", value),
                                                "type": value.type_name(),
                                                "variablesReference": 0
                                            })),
                                        },
                                        Err(e) => Response {
                                            seq: self.next_seq(),
                                            msg_type: "response".to_string(),
                                            request_seq: request.seq,
                                            success: false,
                                            command: "evaluate".to_string(),
                                            message: Some(crate::i18n::fmt_msg(crate::i18n::MsgKey::DapEvaluationError, &[&e])),
                                            body: None,
                                        },
                                    }
                                }
                                Err(e) => Response {
                                    seq: self.next_seq(),
                                    msg_type: "response".to_string(),
                                    request_seq: request.seq,
                                    success: false,
                                    command: "evaluate".to_string(),
                                    message: Some(crate::i18n::fmt_msg(crate::i18n::MsgKey::DapParseError, &[&e])),
                                    body: None,
                                },
                            }
                        } else {
                            Response {
                                seq: self.next_seq(),
                                msg_type: "response".to_string(),
                                request_seq: request.seq,
                                success: false,
                                command: "evaluate".to_string(),
                                message: Some(crate::i18n::fmt_msg(crate::i18n::MsgKey::DapNoEnvironment, &[])),
                                body: None,
                            }
                        }
                    } else {
                        Response {
                            seq: self.next_seq(),
                            msg_type: "response".to_string(),
                            request_seq: request.seq,
                            success: false,
                            command: "evaluate".to_string(),
                            message: Some(crate::i18n::fmt_msg(crate::i18n::MsgKey::DapDebuggerNotAvailable, &[])),
                            body: None,
                        }
                    }
                }
            }
            None => Response {
                seq: self.next_seq(),
                msg_type: "response".to_string(),
                request_seq: request.seq,
                success: false,
                command: "evaluate".to_string(),
                message: Some("Missing 'expression' argument".to_string()),
                body: None,
            },
        }
    }

    /// writeStdinリクエストハンドラー（標準入力エミュレート）
    ///
    /// クライアントから送られたテキストをQiプログラムのstdinに書き込みます。
    /// 引数: { "text": "入力文字列" }
    fn handle_write_stdin(&self, request: Request) -> Response {
        let text = request
            .arguments
            .as_ref()
            .and_then(|args| args.get("text"))
            .and_then(|v| v.as_str());

        match text {
            Some(text_str) => match write_to_stdin(text_str) {
                Ok(()) => Response {
                    seq: self.next_seq(),
                    msg_type: "response".to_string(),
                    request_seq: request.seq,
                    success: true,
                    command: "writeStdin".to_string(),
                    message: None,
                    body: None,
                },
                Err(e) => Response {
                    seq: self.next_seq(),
                    msg_type: "response".to_string(),
                    request_seq: request.seq,
                    success: false,
                    command: "writeStdin".to_string(),
                    message: Some(format!("Failed to write to stdin: {}", e)),
                    body: None,
                },
            },
            None => Response {
                seq: self.next_seq(),
                msg_type: "response".to_string(),
                request_seq: request.seq,
                success: false,
                command: "writeStdin".to_string(),
                message: Some("Missing 'text' argument".to_string()),
                body: None,
            },
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
/// ```text
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
// DAPサーバーのログ出力（元のstderrに書き込む）
// ========================================

use parking_lot::Mutex;
use std::sync::LazyLock;

/// 元のstderr（プログラム実行時のリダイレクトの影響を受けない）
#[cfg(unix)]
static ORIGINAL_STDERR: LazyLock<Mutex<Option<i32>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(windows)]
static ORIGINAL_STDERR: LazyLock<Mutex<Option<windows_sys::Win32::Foundation::HANDLE>>> =
    LazyLock::new(|| Mutex::new(None));

/// DAPサーバーのログを元のstderrに出力
fn dap_log(message: &str) {
    if let Some(fd) = *ORIGINAL_STDERR.lock() {
        let msg = format!("{}\n", message);

        #[cfg(unix)]
        unsafe {
            libc::write(fd, msg.as_ptr() as *const _, msg.len());
        }

        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::Storage::FileSystem::*;
            let mut written: u32 = 0;
            WriteFile(
                fd,
                msg.as_ptr() as *const _,
                msg.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }
}

// ========================================
// DAPサーバー用ヘルパー関数
// ========================================

/// 元のstdoutをバックアップしてFileを作成
#[cfg(unix)]
fn backup_stdout() -> io::Result<std::fs::File> {
    unsafe {
        let fd = libc::dup(libc::STDOUT_FILENO);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        use std::os::unix::io::FromRawFd;
        Ok(std::fs::File::from_raw_fd(fd))
    }
}

#[cfg(windows)]
fn backup_stdout() -> io::Result<std::fs::File> {
    unsafe {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::System::Console::*;
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        Ok(std::fs::File::from_raw_handle(handle as _))
    }
}

/// Qiプログラム用のstdin書き込み側fd（将来のinputイベント用）
static QI_STDIN_WRITE_FD: parking_lot::Mutex<Option<i32>> = parking_lot::Mutex::new(None);

/// 元のstdin (fd 0)を保存し、Qiプログラム用のパイプに差し替える
///
/// 戻り値: DAPサーバーが使用する元のstdin
#[cfg(unix)]
unsafe fn backup_stdin() -> io::Result<std::fs::File> {
    use std::os::unix::io::FromRawFd;
    // 1. 元の stdin (fd 0) を複製して保存
    let original_stdin_fd = libc::dup(libc::STDIN_FILENO);
    if original_stdin_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    // 2. Qiプログラム用のパイプを作成
    let mut pipe_fds = [0i32; 2];
    if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
        libc::close(original_stdin_fd);
        return Err(io::Error::last_os_error());
    }
    let pipe_read = pipe_fds[0];
    let pipe_write = pipe_fds[1];

    // 3. fd 0 をパイプの read 側に差し替える
    if libc::dup2(pipe_read, libc::STDIN_FILENO) < 0 {
        libc::close(original_stdin_fd);
        libc::close(pipe_read);
        libc::close(pipe_write);
        return Err(io::Error::last_os_error());
    }

    // 4. パイプの read 側は不要（fd 0 に複製済み）
    libc::close(pipe_read);

    // 5. パイプの write 側を保存（input イベントで使用）
    *QI_STDIN_WRITE_FD.lock() = Some(pipe_write);

    // 6. 元の stdin fd を File に変換して返す
    Ok(std::fs::File::from_raw_fd(original_stdin_fd))
}

#[cfg(windows)]
unsafe fn backup_stdin() -> io::Result<std::fs::File> {
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::System::Console::*;
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        DuplicateHandle, GetCurrentProcess, DUPLICATE_SAME_ACCESS,
    };

    // 1. 元の stdin ハンドルを取得
    let original_stdin = GetStdHandle(STD_INPUT_HANDLE);
    if original_stdin == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }

    // 2. stdin ハンドルを複製して保存
    let mut duplicated_stdin: HANDLE = 0;
    if DuplicateHandle(
        GetCurrentProcess(),
        original_stdin,
        GetCurrentProcess(),
        &mut duplicated_stdin,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    ) == 0
    {
        return Err(io::Error::last_os_error());
    }

    // 3. Qiプログラム用のパイプを作成
    let mut pipe_read: HANDLE = 0;
    let mut pipe_write: HANDLE = 0;
    if CreatePipe(&mut pipe_read, &mut pipe_write, std::ptr::null(), 0) == 0 {
        CloseHandle(duplicated_stdin);
        return Err(io::Error::last_os_error());
    }

    // 4. stdin をパイプの read 側に差し替え
    if SetStdHandle(STD_INPUT_HANDLE, pipe_read) == 0 {
        CloseHandle(duplicated_stdin);
        CloseHandle(pipe_read);
        CloseHandle(pipe_write);
        return Err(io::Error::last_os_error());
    }

    // 5. パイプの write 側を保存（input イベントで使用）
    *QI_STDIN_WRITE_FD.lock() = Some(pipe_write as i32);

    // 6. 複製した元の stdin ハンドルを File に変換して返す
    Ok(std::fs::File::from_raw_handle(duplicated_stdin as _))
}

/// Qiプログラムのstdinにテキストを書き込む（DAPのwriteStdinリクエスト用）
///
/// 引数のテキストを改行付きでパイプに書き込みます。
#[cfg(unix)]
fn write_to_stdin(text: &str) -> io::Result<()> {
    let fd_guard = QI_STDIN_WRITE_FD.lock();
    if let Some(write_fd) = *fd_guard {
        let data = format!("{}\n", text);
        unsafe {
            let bytes_written =
                libc::write(write_fd, data.as_ptr() as *const libc::c_void, data.len());
            if bytes_written < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "stdin write fd not available",
        ))
    }
}

#[cfg(windows)]
fn write_to_stdin(text: &str) -> io::Result<()> {
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Storage::FileSystem::WriteFile;

    let fd_guard = QI_STDIN_WRITE_FD.lock();
    if let Some(handle) = *fd_guard {
        let data = format!("{}\n", text);
        let mut bytes_written: u32 = 0;
        unsafe {
            if WriteFile(
                handle as HANDLE,
                data.as_ptr(),
                data.len() as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "stdin write handle not available",
        ))
    }
}

/// 元のstderrをバックアップ（DAPログ出力用）
#[cfg(unix)]
fn backup_stderr_for_logging() {
    unsafe {
        let fd = libc::dup(libc::STDERR_FILENO);
        if fd >= 0 {
            *ORIGINAL_STDERR.lock() = Some(fd);
        }
    }
}

#[cfg(windows)]
fn backup_stderr_for_logging() {
    unsafe {
        use windows_sys::Win32::System::Console::*;
        let handle = GetStdHandle(STD_ERROR_HANDLE);
        *ORIGINAL_STDERR.lock() = Some(handle);
    }
}

// ========================================
// DAPサーバーメインループ
// ========================================

impl DapServer {
    /// DAPサーバーを起動（非同期版・推奨）
    pub async fn run_async_internal() -> io::Result<()> {
        let server = std::sync::Arc::new(DapServer::new());

        // 元の stdin (fd 0) を保存してから、Qi プログラム用のパイプに差し替える
        let original_stdin_file = unsafe { backup_stdin()? };
        let stdin = tokio::fs::File::from_std(original_stdin_file);

        // 元のstdout/stderrをバックアップ
        let stdout_file = backup_stdout()?;
        backup_stderr_for_logging();

        let mut writer = tokio::io::BufWriter::new(tokio::fs::File::from_std(stdout_file));
        let mut reader = AsyncBufReader::new(stdin);

        dap_log("[DAP] Qi Debug Adapter starting (async mode)...");
        eprintln!("[DAP] Log file: /tmp/qi-dap.log");

        let exe_path = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());

        std::fs::write(
            "/tmp/qi-dap.log",
            format!(
                "DAP server started at {:?}\nExecutable: {}\n",
                std::time::SystemTime::now(),
                exe_path
            ),
        )
        .ok();

        // デバッガーを初期化
        crate::debugger::init_global_debugger(true);

        // イベント送信用のchannel
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<String>(100);

        // 停止イベント監視用のタイマー
        let mut stopped_event_interval =
            tokio::time::interval(tokio::time::Duration::from_millis(50));

        // 前回送信したイベント（重複送信防止）
        let mut last_sent_event: Option<(String, usize, usize, String)> = None;

        // メインループ
        loop {
            tokio::select! {
                // 停止イベントの監視（50msごと）
                _ = stopped_event_interval.tick() => {
                    let event_info = {
                        let guard = crate::debugger::GLOBAL_DEBUGGER.read();
                        if let Some(ref dbg) = *guard {
                            dbg.get_stopped_event().cloned()
                        } else {
                            None
                        }
                    };

                    // 前回と同じイベントは送信しない
                    if event_info.is_some() && event_info == last_sent_event {
                        continue;
                    }

                    if let Some((file, line, column, reason)) = event_info.clone() {
                        last_sent_event = event_info;
                        let log_msg = format!(
                            "[DAP] Sending stopped event: reason={}, file={}, line={}, column={}\n",
                            reason, file, line, column
                        );
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();

                        let body = serde_json::json!({
                            "reason": reason,
                            "threadId": 1,
                            "allThreadsStopped": true,
                        });

                        let event = server.send_event("stopped", Some(body));
                        if let Ok(event_json) = serde_json::to_string(&event) {
                            write_message_async(&mut writer, &event_json).await?;
                        }
                    }
                }
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

                            // ログに記録
                            let log_msg = format!("Request: {}\n", request.command);
                            std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("/tmp/qi-dap.log")
                                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                                .ok();

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
        let server = Arc::new(DapServer::new());
        let stdout = Arc::new(parking_lot::Mutex::new(io::stdout()));

        // 元の stdin (fd 0) を保存してから、Qi プログラム用のパイプに差し替える
        let original_stdin_file = unsafe { backup_stdin()? };
        let mut reader = BufReader::new(original_stdin_file);

        // 起動ログ
        let exe_path = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());

        dap_log(&format!("[DAP] Qi Debug Adapter starting (sync mode)..."));
        dap_log(&format!("[DAP] Executable: {}", exe_path));

        // デバッガーを初期化
        crate::debugger::init_global_debugger(true);

        // イベント監視スレッドを起動
        let server_clone = Arc::clone(&server);
        let stdout_clone = Arc::clone(&stdout);
        std::thread::spawn(move || {
            // 前回送信したイベント（重複送信防止）
            let mut last_sent_event: Option<(String, usize, usize, String)> = None;

            loop {
                // 50ms ごとに stopped イベントをチェック
                std::thread::sleep(std::time::Duration::from_millis(50));

                // stopped イベントをチェック
                let event_info = {
                    let guard = crate::debugger::GLOBAL_DEBUGGER.read();
                    if let Some(ref dbg) = *guard {
                        dbg.get_stopped_event().cloned()
                    } else {
                        None
                    }
                };

                // 前回と同じイベントは送信しない
                if event_info.is_some() && event_info == last_sent_event {
                    continue;
                }

                // イベントがあれば送信
                if let Some((file, line, column, reason)) = event_info.clone() {
                    last_sent_event = event_info;
                    let log_msg = format!(
                        "[DAP] Sending stopped event: reason={}, file={}, line={}, column={}\n",
                        reason, file, line, column
                    );
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

                    let body = serde_json::json!({
                        "reason": reason,
                        "threadId": 1,
                        "allThreadsStopped": true,
                    });

                    let event = server_clone.send_event("stopped", Some(body));
                    if let Ok(event_json) = serde_json::to_string(&event) {
                        let mut stdout_guard = stdout_clone.lock();
                        let _ = write_message(&mut *stdout_guard, &event_json);
                    }
                }
            }
        });

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

            {
                let mut stdout_guard = stdout.lock();
                write_message(&mut *stdout_guard, &response_json)?;
            }

            // initialized イベント送信（initializeリクエスト後）
            if response.command == "initialize" && response.success {
                let initialized_event = server.create_initialized_event();
                let event_json = serde_json::to_string(&initialized_event)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                let mut stdout_guard = stdout.lock();
                write_message(&mut *stdout_guard, &event_json)?;
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
    use std::io;

    // プラットフォーム固有の型定義
    #[cfg(unix)]
    type NativeHandle = i32;

    #[cfg(windows)]
    type NativeHandle = windows_sys::Win32::Foundation::HANDLE;

    // 統一された構造体定義
    pub struct StdioRedirect {
        original_stdout: NativeHandle,
        original_stderr: NativeHandle,
        stdout_read: NativeHandle,
        stderr_read: NativeHandle,
    }

    // プラットフォーム固有のヘルパー関数
    #[cfg(unix)]
    mod platform {
        use super::NativeHandle;
        use std::io;

        pub const STDOUT_NO: NativeHandle = libc::STDOUT_FILENO;
        pub const STDERR_NO: NativeHandle = libc::STDERR_FILENO;

        pub unsafe fn dup(handle: NativeHandle) -> io::Result<NativeHandle> {
            let new_handle = libc::dup(handle);
            if new_handle < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(new_handle)
            }
        }

        pub unsafe fn close(handle: NativeHandle) {
            libc::close(handle);
        }

        pub unsafe fn create_pipe() -> io::Result<(NativeHandle, NativeHandle)> {
            let mut pipe: [i32; 2] = [0, 0];
            if libc::pipe(pipe.as_mut_ptr()) < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok((pipe[0], pipe[1]))
            }
        }

        pub unsafe fn redirect(new_handle: NativeHandle, std_no: NativeHandle) -> io::Result<()> {
            if libc::dup2(new_handle, std_no) < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        /// ハンドルを複製（リーダー用）
        pub unsafe fn dup_for_reader(handle: NativeHandle) -> Option<NativeHandle> {
            let new_handle = libc::dup(handle);
            if new_handle < 0 {
                None
            } else {
                Some(new_handle)
            }
        }

        /// ハンドルからFileリーダーを作成
        pub unsafe fn create_reader(handle: NativeHandle) -> std::fs::File {
            use std::os::unix::io::FromRawFd;
            std::fs::File::from_raw_fd(handle)
        }
    }

    #[cfg(windows)]
    mod platform {
        use super::NativeHandle;
        use std::io;
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Console::*;
        use windows_sys::Win32::System::Pipes::*;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        pub const STDOUT_NO: u32 = STD_OUTPUT_HANDLE;
        pub const STDERR_NO: u32 = STD_ERROR_HANDLE;

        pub unsafe fn get_std_handle(handle_id: u32) -> io::Result<NativeHandle> {
            let handle = GetStdHandle(handle_id);
            if handle == INVALID_HANDLE_VALUE {
                Err(io::Error::last_os_error())
            } else {
                Ok(handle)
            }
        }

        pub unsafe fn close(handle: NativeHandle) {
            CloseHandle(handle);
        }

        pub unsafe fn create_pipe() -> io::Result<(NativeHandle, NativeHandle)> {
            let mut read_handle: HANDLE = 0;
            let mut write_handle: HANDLE = 0;
            if CreatePipe(&mut read_handle, &mut write_handle, std::ptr::null(), 0) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok((read_handle, write_handle))
            }
        }

        pub unsafe fn redirect(new_handle: NativeHandle, std_handle_id: u32) -> io::Result<()> {
            if SetStdHandle(std_handle_id, new_handle) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        /// ハンドルを複製（リーダー用）
        pub unsafe fn dup_for_reader(handle: NativeHandle) -> Option<NativeHandle> {
            let mut dup_handle: HANDLE = 0;
            if DuplicateHandle(
                GetCurrentProcess(),
                handle,
                GetCurrentProcess(),
                &mut dup_handle,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ) == 0
            {
                None
            } else {
                Some(dup_handle)
            }
        }

        /// ハンドルからFileリーダーを作成
        pub unsafe fn create_reader(handle: NativeHandle) -> std::fs::File {
            use std::os::windows::io::FromRawHandle;
            std::fs::File::from_raw_handle(handle as _)
        }
    }

    impl StdioRedirect {
        /// stdout/stderrをパイプにリダイレクト（Unix版）
        #[cfg(unix)]
        pub fn new() -> io::Result<Self> {
            use platform::*;

            unsafe {
                // 元のstdout/stderrを保存
                let original_stdout = dup(STDOUT_NO)?;
                let original_stderr = dup(STDERR_NO).inspect_err(|_e| {
                    close(original_stdout);
                })?;

                // パイプ作成（stdout、stderr）
                let (stdout_read, stdout_write) = create_pipe().inspect_err(|_e| {
                    close(original_stdout);
                    close(original_stderr);
                })?;

                let (stderr_read, stderr_write) = create_pipe().inspect_err(|_e| {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stdout_write);
                })?;

                // リダイレクト
                if let Err(e) = redirect(stdout_write, STDOUT_NO) {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stdout_write);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                if let Err(e) = redirect(stderr_write, STDERR_NO) {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                // 書き込み側を閉じる
                close(stdout_write);
                close(stderr_write);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read,
                    stderr_read,
                })
            }
        }

        /// stdout/stderrをパイプにリダイレクト（Windows版）
        #[cfg(windows)]
        pub fn new() -> io::Result<Self> {
            use platform::*;
            use windows_sys::Win32::Storage::FileSystem::*;

            unsafe {
                // 元のstdout/stderrを保存
                let original_stdout = get_std_handle(STDOUT_NO)?;
                let original_stderr = get_std_handle(STDERR_NO)?;

                // パイプ作成（stdout、stderr）
                let (stdout_read, stdout_write) = create_pipe()?;

                let (stderr_read, stderr_write) = create_pipe().map_err(|e| {
                    close(stdout_read);
                    close(stdout_write);
                    e
                })?;

                // リダイレクト
                if let Err(e) = redirect(stdout_write, STDOUT_NO) {
                    close(stdout_read);
                    close(stdout_write);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                if let Err(e) = redirect(stderr_write, STDERR_NO) {
                    close(stdout_read);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                // 書き込み側を閉じる
                close(stdout_write);
                close(stderr_write);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read,
                    stderr_read,
                })
            }
        }

        /// stdoutパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stdout_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(event_tx, seq_base, "stdout", self.stdout_read)
        }

        /// stderrパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stderr_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(event_tx, seq_base, "stderr", self.stderr_read)
        }

        /// パイプから読み取ってDAPイベントを送信する共通実装
        fn spawn_reader_impl(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
            category: &'static str,
            read_handle: NativeHandle,
        ) -> tokio::task::JoinHandle<()> {
            // ハンドルを複製
            let read_dup = unsafe { platform::dup_for_reader(read_handle) };

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;

                // 複製に失敗した場合は早期リターン
                let Some(handle) = read_dup else {
                    return;
                };

                // リーダーを作成
                let file = unsafe { platform::create_reader(handle) };
                let async_file = tokio::fs::File::from_std(file);
                let mut reader = tokio::io::BufReader::new(async_file);

                let mut line = String::new();
                let mut seq = seq_base;
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let output_msg = OutputEventBody {
                                category: category.to_string(),
                                output: line.clone(),
                            };
                            let output_event = Event {
                                seq,
                                msg_type: "event".to_string(),
                                event: "output".to_string(),
                                body: serde_json::to_value(&output_msg).ok(),
                            };
                            if let Ok(event_json) = serde_json::to_string(&output_event) {
                                let _ = event_tx.send(event_json).await;
                            }
                            seq += 1;
                        }
                        Err(_) => break,
                    }
                }
            })
        }
    }

    impl Drop for StdioRedirect {
        /// stdout/stderrを元に戻す
        fn drop(&mut self) {
            use platform::*;

            unsafe {
                // 元のstdout/stderrを復元
                let _ = redirect(self.original_stdout, STDOUT_NO);
                let _ = redirect(self.original_stderr, STDERR_NO);

                // ハンドルをクローズ
                close(self.original_stdout);
                close(self.original_stderr);
                close(self.stdout_read);
                close(self.stderr_read);
            }
        }
    }
}
