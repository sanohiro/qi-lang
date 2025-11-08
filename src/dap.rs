//! Debug Adapter Protocol (DAP) サーバー実装
//!
//! VSCodeとの統合のためのDAPサーバー

use crate::constants::dap::*;
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

/// DAPサーバーのコンテキスト（状態管理）
struct DapContext {
    server: Arc<DapServer>,
    writer: tokio::io::BufWriter<tokio::fs::File>,
    reader: AsyncBufReader<tokio::fs::File>,
    event_tx: tokio::sync::mpsc::Sender<String>,
    event_rx: tokio::sync::mpsc::Receiver<String>,
    stopped_event_interval: tokio::time::Interval,
    last_sent_event: Option<(String, usize, usize, String)>,
}

impl DapContext {
    /// DAPコンテキストを初期化
    async fn new() -> io::Result<Self> {
        // stdin/stdoutのバックアップ
        let original_stdin_file = unsafe { backup_stdin()? };
        let stdin = tokio::fs::File::from_std(original_stdin_file);

        let stdout_file = backup_stdout()?;
        backup_stderr_for_logging();

        let writer = tokio::io::BufWriter::new(tokio::fs::File::from_std(stdout_file));
        let reader = AsyncBufReader::new(stdin);

        // イベント送信用のchannel
        let (event_tx, event_rx) = tokio::sync::mpsc::channel::<String>(100);

        // 停止イベント監視用のタイマー
        let stopped_event_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

        Ok(Self {
            server: Arc::new(DapServer::new()),
            writer,
            reader,
            event_tx,
            event_rx,
            stopped_event_interval,
            last_sent_event: None,
        })
    }

    /// 起動ログを記録
    fn log_startup(&self) {
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
    }

    /// デバッガーを初期化
    fn initialize_debugger(&self) {
        crate::debugger::init_global_debugger(true);
    }

    /// メインループを実行
    async fn run_main_loop(&mut self) -> io::Result<()> {
        loop {
            tokio::select! {
                _ = self.stopped_event_interval.tick() => {
                    self.handle_stopped_event_tick().await?;
                }
                message_result = read_message_async(&mut self.reader) => {
                    if !self.handle_incoming_message(message_result).await? {
                        break;
                    }
                }
                Some(event_json) = self.event_rx.recv() => {
                    self.send_event(&event_json).await?;
                }
            }
        }
        Ok(())
    }

    /// 停止イベントのチェック
    async fn handle_stopped_event_tick(&mut self) -> io::Result<()> {
        let event_info = self.get_stopped_event_info();

        // 前回と同じイベントは送信しない
        if event_info.is_some() && event_info == self.last_sent_event {
            return Ok(());
        }

        if let Some((file, line, column, reason)) = event_info.clone() {
            self.last_sent_event = event_info;
            self.log_stopped_event(&reason, &file, line, column);

            let body = serde_json::json!({
                "reason": reason,
                "threadId": 1,
                "allThreadsStopped": true,
            });

            let event = self.server.send_event(EVENT_STOPPED, Some(body));
            if let Ok(event_json) = serde_json::to_string(&event) {
                write_message_async(&mut self.writer, &event_json).await?;
            }
        }

        Ok(())
    }

    /// 停止イベント情報を取得
    fn get_stopped_event_info(&self) -> Option<(String, usize, usize, String)> {
        let guard = crate::debugger::GLOBAL_DEBUGGER.read();
        if let Some(ref dbg) = *guard {
            dbg.get_stopped_event().cloned()
        } else {
            None
        }
    }

    /// 停止イベントのログ出力
    fn log_stopped_event(&self, reason: &str, file: &str, line: usize, column: usize) {
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
    }

    /// 受信メッセージの処理（戻り値: true=継続, false=終了）
    async fn handle_incoming_message(
        &mut self,
        message_result: io::Result<String>,
    ) -> io::Result<bool> {
        match message_result {
            Ok(message) => {
                self.process_request(&message).await?;
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
            Err(_) => Ok(true),
        }
    }

    /// リクエストの処理
    async fn process_request(&mut self, message: &str) -> io::Result<()> {
        // JSONパース
        let request: Request = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(_) => return Ok(()),
        };

        // ログに記録
        self.log_request(&request.command);

        // リクエスト処理
        let response = self
            .server
            .handle_request_async(request.clone(), self.event_tx.clone());

        // レスポンス送信
        let response_json = serde_json::to_string(&response)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        write_message_async(&mut self.writer, &response_json).await?;

        // initialized イベント送信
        if response.command == COMMAND_INITIALIZE && response.success {
            self.send_initialized_event().await?;
        }

        Ok(())
    }

    /// リクエストのログ出力
    fn log_request(&self, command: &str) {
        let log_msg = format!("Request: {}\n", command);
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();
    }

    /// initialized イベント送信
    async fn send_initialized_event(&mut self) -> io::Result<()> {
        let initialized_event = self.server.create_initialized_event();
        let event_json = serde_json::to_string(&initialized_event)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_message_async(&mut self.writer, &event_json).await
    }

    /// イベント送信
    async fn send_event(&mut self, event_json: &str) -> io::Result<()> {
        write_message_async(&mut self.writer, event_json).await
    }
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
            msg_type: MSG_TYPE_EVENT.to_string(),
            event: event_name.to_string(),
            body,
        }
    }

    pub fn handle_request(&self, request: Request) -> Response {
        match request.command.as_str() {
            COMMAND_INITIALIZE => self.handle_initialize(request),
            COMMAND_LAUNCH => self.handle_launch(request),
            COMMAND_ATTACH => self.handle_attach(request),
            COMMAND_SET_BREAKPOINTS => self.handle_set_breakpoints(request),
            COMMAND_CONFIGURATION_DONE => self.handle_configuration_done(request),
            COMMAND_THREADS => self.handle_threads(request),
            COMMAND_STACK_TRACE => self.handle_stack_trace(request),
            COMMAND_SCOPES => self.handle_scopes(request),
            COMMAND_VARIABLES => self.handle_variables(request),
            COMMAND_CONTINUE => self.handle_continue(request),
            COMMAND_NEXT => self.handle_next(request),
            COMMAND_STEP_IN => self.handle_step_in(request),
            COMMAND_STEP_OUT => self.handle_step_out(request),
            COMMAND_DISCONNECT => self.handle_disconnect(request),
            "writeStdin" => self.handle_write_stdin(request),
            COMMAND_EVALUATE => self.handle_evaluate(request),
            _ => self.create_error_response(
                request.seq,
                &request.command,
                format!("Unknown command: {}", request.command),
            ),
        }
    }

    /// 非同期版リクエストハンドラ（イベント送信チャネル付き）
    pub fn handle_request_async(
        &self,
        request: Request,
        event_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Response {
        match request.command.as_str() {
            COMMAND_LAUNCH => self.handle_launch_async(request, event_tx),
            COMMAND_CONFIGURATION_DONE => self.handle_configuration_done_async(request, event_tx),
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

                    return self.create_success_response(request.seq, COMMAND_LAUNCH, None);
                } else {
                    return self.create_error_response(
                        request.seq,
                        COMMAND_LAUNCH,
                        "No program specified".to_string(),
                    );
                }
            }
        }

        self.create_error_response(
            request.seq,
            COMMAND_LAUNCH,
            "Invalid launch arguments".to_string(),
        )
    }

    fn handle_launch(&self, request: Request) -> Response {
        // launchの引数を解析
        if let Some(args) = request.arguments {
            if let Ok(launch_args) = serde_json::from_value::<LaunchRequestArguments>(args) {
                if let Some(_program) = launch_args.program {
                    // NOTE: この同期版handle_launchは非推奨。run_async()を使用すること
                    // プログラム実行は非同期版（handle_launch_async）で実装済み
                } else {
                    return self.create_error_response(
                        request.seq,
                        COMMAND_LAUNCH,
                        "No program specified".to_string(),
                    );
                }
            }
        }

        self.create_success_response(request.seq, COMMAND_LAUNCH, None)
    }

    fn handle_attach(&self, request: Request) -> Response {
        self.create_success_response(request.seq, COMMAND_ATTACH, None)
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

        self.create_success_response(
            request.seq,
            COMMAND_INITIALIZE,
            Some(serde_json::to_value(capabilities).unwrap()),
        )
    }

    fn handle_set_breakpoints(&self, request: Request) -> Response {
        let log_msg = "setBreakpoints called\n".to_string();
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

                return self.create_success_response(
                    request.seq,
                    COMMAND_SET_BREAKPOINTS,
                    Some(body),
                );
            }
        }

        self.create_error_response(
            request.seq,
            COMMAND_SET_BREAKPOINTS,
            "Invalid arguments".to_string(),
        )
    }

    fn handle_threads(&self, request: Request) -> Response {
        let threads = vec![Thread {
            id: 1,
            name: "Main Thread".to_string(),
        }];

        let body = serde_json::json!({ "threads": threads });

        self.create_success_response(request.seq, COMMAND_THREADS, Some(body))
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

        self.create_success_response(request.seq, COMMAND_STACK_TRACE, Some(body))
    }

    fn handle_scopes(&self, _request: Request) -> Response {
        // Localスコープとグローバルスコープに対応
        let scopes = vec![
            Scope {
                name: "Local".to_string(),
                variables_reference: 1, // ローカル変数用の参照ID
                expensive: false,
            },
            Scope {
                name: "Global".to_string(),
                variables_reference: 2, // グローバル変数用の参照ID
                expensive: true,        // グローバルは大きい可能性があるのでexpensive
            },
        ];

        let body = serde_json::json!({ "scopes": scopes });

        self.create_success_response(_request.seq, COMMAND_SCOPES, Some(body))
    }

    fn handle_variables(&self, request: Request) -> Response {
        use crate::value::Value;

        // argumentsからvariables_referenceを取得
        let variables_reference: i64 = request
            .arguments
            .as_ref()
            .and_then(|args| args.get("variablesReference"))
            .and_then(|v| v.as_i64())
            .unwrap_or(1);

        let mut variables: Vec<Variable> = vec![];

        if let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() {
            if let Some(env_arc) = dbg.get_stopped_env() {
                let env = env_arc.read();

                // variables_reference に応じてスコープを切り替え
                match variables_reference {
                    1 => {
                        // ローカル変数
                        for (name, binding) in env.local_bindings() {
                            let is_callable = matches!(
                                binding.value,
                                Value::Function(_) | Value::Macro(_) | Value::NativeFunc(_)
                            );

                            if !is_callable {
                                Self::push_variable(&mut variables, name, &binding.value);
                            }
                        }
                    }
                    2 => {
                        // グローバル変数（全バインディングから取得）
                        for (name, binding) in env.all_bindings() {
                            let is_callable = matches!(
                                binding.value,
                                Value::Function(_) | Value::Macro(_) | Value::NativeFunc(_)
                            );

                            if !is_callable {
                                Self::push_variable(&mut variables, name.clone(), &binding.value);
                            }
                        }
                    }
                    _ => {
                        // 今のところネストした値の展開は未対応（将来的に実装予定）
                    }
                }
            }
        }

        let body = serde_json::json!({ "variables": variables });

        self.create_success_response(request.seq, COMMAND_VARIABLES, Some(body))
    }

    /// 変数を変数リストに追加するヘルパー関数
    fn push_variable(variables: &mut Vec<Variable>, name: String, value: &crate::value::Value) {
        use crate::value::Value;

        // ネストした値（map/vector/list）の場合はvariables_referenceを設定
        let (value_str, var_ref) = match value {
            Value::Map(m) if !m.is_empty() => {
                (format!("Map({})", m.len()), 0) // 将来的に展開対応予定
            }
            Value::Vector(v) if !v.is_empty() => {
                (format!("Vector[{}]", v.len()), 0) // 将来的に展開対応予定
            }
            Value::List(l) if !l.is_empty() => {
                (format!("List({})", l.len()), 0) // 将来的に展開対応予定
            }
            _ => (format!("{}", value), 0),
        };

        variables.push(Variable {
            name,
            value: value_str,
            var_type: Some(value.type_name().to_string()),
            variables_reference: var_ref,
        });
    }

    fn handle_continue(&self, request: Request) -> Response {
        // デバッガを再開
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.resume();
        }

        let body = serde_json::json!({ "allThreadsContinued": true });

        self.create_success_response(request.seq, COMMAND_CONTINUE, Some(body))
    }

    fn handle_next(&self, request: Request) -> Response {
        // ステップオーバー
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_over();
            dbg.resume(); // 待機中のスレッドを起こす
        }

        self.create_success_response(request.seq, COMMAND_NEXT, None)
    }

    fn handle_step_in(&self, request: Request) -> Response {
        // ステップイン
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_in();
            dbg.resume(); // 待機中のスレッドを起こす
        }

        self.create_success_response(request.seq, COMMAND_STEP_IN, None)
    }

    fn handle_step_out(&self, request: Request) -> Response {
        // ステップアウト
        if let Some(ref mut dbg) = *crate::debugger::GLOBAL_DEBUGGER.write() {
            dbg.step_out();
            dbg.resume(); // 待機中のスレッドを起こす
        }

        self.create_success_response(request.seq, COMMAND_STEP_OUT, None)
    }

    fn handle_configuration_done(&self, request: Request) -> Response {
        self.create_success_response(request.seq, COMMAND_CONFIGURATION_DONE, None)
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
                    msg_type: MSG_TYPE_EVENT.to_string(),
                    event: EVENT_OUTPUT.to_string(),
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
                            msg_type: MSG_TYPE_EVENT.to_string(),
                            event: EVENT_OUTPUT.to_string(),
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
                            msg_type: MSG_TYPE_EVENT.to_string(),
                            event: EVENT_OUTPUT.to_string(),
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
                    msg_type: MSG_TYPE_EVENT.to_string(),
                    event: EVENT_TERMINATED.to_string(),
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

        self.create_success_response(request.seq, COMMAND_CONFIGURATION_DONE, None)
    }

    fn handle_disconnect(&self, request: Request) -> Response {
        self.create_success_response(request.seq, COMMAND_DISCONNECT, None)
    }

    /// evaluateリクエストハンドラー（デバッグコンソール入力）
    ///
    /// デバッグコンソールから入力された式を評価します。
    /// `.stdin <text>` 形式の入力を標準入力に書き込みます。
    /// 引数: { "expression": "式またはコマンド", "frameId": ..., "context": ... }
    fn handle_evaluate(&self, request: Request) -> Response {
        let Some(expr) = self.extract_expression(&request) else {
            return self.create_error_response(
                request.seq,
                COMMAND_EVALUATE,
                "Missing 'expression' argument".to_string(),
            );
        };

        // .stdinコマンドの処理
        if let Some(text) = expr.strip_prefix(".stdin ") {
            return self.handle_stdin_command(&request, text);
        }

        // ウォッチ式評価
        self.handle_watch_expression(&request, expr)
    }

    /// リクエストから式を抽出
    fn extract_expression<'a>(&self, request: &'a Request) -> Option<&'a str> {
        request
            .arguments
            .as_ref()
            .and_then(|args| args.get("expression"))
            .and_then(|v| v.as_str())
    }

    /// .stdinコマンドの処理
    fn handle_stdin_command(&self, request: &Request, text: &str) -> Response {
        let text = text.replace("\\n", "\n").replace("\\t", "\t");

        match write_to_stdin(&text) {
            Ok(()) => self.create_success_response(
                request.seq,
                COMMAND_EVALUATE,
                Some(serde_json::json!({
                    "result": crate::i18n::fmt_ui_msg(crate::i18n::UiMsg::DapStdinSent, &[&text]),
                    "variablesReference": 0
                })),
            ),
            Err(e) => self.create_error_response(
                request.seq,
                COMMAND_EVALUATE,
                format!("Failed to write to stdin: {}", e),
            ),
        }
    }

    /// ウォッチ式の評価
    fn handle_watch_expression(&self, request: &Request, expr: &str) -> Response {
        // デバッガーの取得
        let Some(ref dbg) = *crate::debugger::GLOBAL_DEBUGGER.read() else {
            return self.create_i18n_error_response(
                request.seq,
                COMMAND_EVALUATE,
                crate::i18n::MsgKey::DapDebuggerNotAvailable,
                &[],
            );
        };

        // 環境の取得
        let Some(env_arc) = dbg.get_stopped_env() else {
            return self.create_i18n_error_response(
                request.seq,
                COMMAND_EVALUATE,
                crate::i18n::MsgKey::DapNoEnvironment,
                &[],
            );
        };

        // パース
        let exprs = match crate::parser::Parser::new(expr).and_then(|mut p| p.parse_all()) {
            Ok(exprs) if !exprs.is_empty() => exprs,
            Ok(_) => {
                return self.create_i18n_error_response(
                    request.seq,
                    COMMAND_EVALUATE,
                    crate::i18n::MsgKey::DapEmptyExpression,
                    &[],
                )
            }
            Err(e) => {
                return self.create_i18n_error_response(
                    request.seq,
                    COMMAND_EVALUATE,
                    crate::i18n::MsgKey::DapParseError,
                    &[&e],
                )
            }
        };

        // 評価
        self.evaluate_expression(request, &exprs[0], env_arc)
    }

    /// 式の評価実行
    fn evaluate_expression(
        &self,
        request: &Request,
        expr: &crate::value::Expr,
        env_arc: std::sync::Arc<parking_lot::RwLock<crate::value::Env>>,
    ) -> Response {
        let evaluator = crate::eval::Evaluator::new();
        match evaluator.eval_with_env(expr, env_arc) {
            Ok(value) => self.create_success_response(
                request.seq,
                COMMAND_EVALUATE,
                Some(serde_json::json!({
                    "result": format!("{}", value),
                    "type": value.type_name(),
                    "variablesReference": 0
                })),
            ),
            Err(e) => self.create_i18n_error_response(
                request.seq,
                COMMAND_EVALUATE,
                crate::i18n::MsgKey::DapEvaluationError,
                &[&e],
            ),
        }
    }

    /// 成功レスポンスの生成
    fn create_success_response(
        &self,
        request_seq: i64,
        command: &str,
        body: Option<serde_json::Value>,
    ) -> Response {
        Response {
            seq: self.next_seq(),
            msg_type: MSG_TYPE_RESPONSE.to_string(),
            request_seq,
            success: true,
            command: command.to_string(),
            message: None,
            body,
        }
    }

    /// エラーレスポンスの生成
    fn create_error_response(&self, request_seq: i64, command: &str, message: String) -> Response {
        Response {
            seq: self.next_seq(),
            msg_type: MSG_TYPE_RESPONSE.to_string(),
            request_seq,
            success: false,
            command: command.to_string(),
            message: Some(message),
            body: None,
        }
    }

    /// i18nエラーレスポンスの生成
    fn create_i18n_error_response(
        &self,
        request_seq: i64,
        command: &str,
        key: crate::i18n::MsgKey,
        args: &[&str],
    ) -> Response {
        self.create_error_response(request_seq, command, crate::i18n::fmt_msg(key, args))
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
                Ok(()) => self.create_success_response(request.seq, "writeStdin", None),
                Err(e) => self.create_error_response(
                    request.seq,
                    "writeStdin",
                    format!("Failed to write to stdin: {}", e),
                ),
            },
            None => self.create_error_response(
                request.seq,
                "writeStdin",
                "Missing 'text' argument".to_string(),
            ),
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
            msg_type: MSG_TYPE_EVENT.to_string(),
            event: EVENT_STOPPED.to_string(),
            body: Some(body),
        }
    }

    pub fn create_initialized_event(&self) -> Event {
        Event {
            seq: self.next_seq(),
            msg_type: MSG_TYPE_EVENT.to_string(),
            event: EVENT_INITIALIZED.to_string(),
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
static ORIGINAL_STDERR: LazyLock<Mutex<Option<stdio_redirect::platform::SendHandle>>> =
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
                fd.0,
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
    use windows_sys::Win32::System::Threading::*;

    // 1. 元の stdin ハンドルを取得
    let original_stdin = GetStdHandle(STD_INPUT_HANDLE);
    if original_stdin == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }

    // 2. stdin ハンドルを複製して保存
    let mut duplicated_stdin: HANDLE = std::ptr::null_mut();
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
        *ORIGINAL_STDERR.lock() = Some(stdio_redirect::platform::SendHandle(handle));
    }
}

// ========================================
// DAPサーバーメインループ
// ========================================

impl DapServer {
    /// DAPサーバーを起動（非同期版・推奨）
    pub async fn run_async_internal() -> io::Result<()> {
        let mut context = DapContext::new().await?;
        context.log_startup();
        context.initialize_debugger();
        context.run_main_loop().await
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

        dap_log("[DAP] Qi Debug Adapter starting (sync mode)...");
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

                    let event = server_clone.send_event(EVENT_STOPPED, Some(body));
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
            if response.command == COMMAND_INITIALIZE && response.success {
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
    use super::{Event, OutputEventBody, EVENT_OUTPUT, MSG_TYPE_EVENT};
    use std::io;

    // プラットフォーム固有の型定義
    #[cfg(unix)]
    type NativeHandle = i32;

    #[cfg(windows)]
    type NativeHandle = windows_sys::Win32::Foundation::HANDLE;

    // 統一された構造体定義
    #[cfg(unix)]
    pub struct StdioRedirect {
        original_stdout: NativeHandle,
        original_stderr: NativeHandle,
        stdout_read: NativeHandle,
        stderr_read: NativeHandle,
    }

    // Windows版はSendHandle使用
    #[cfg(windows)]
    pub struct StdioRedirect {
        original_stdout: platform::SendHandle,
        original_stderr: platform::SendHandle,
        stdout_read: platform::SendHandle,
        stderr_read: platform::SendHandle,
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
    pub(super) mod platform {
        use super::NativeHandle;
        use std::io;
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Console::*;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        /// Windows HANDLEをSend-safeにするラッパー
        #[derive(Clone, Copy)]
        pub struct SendHandle(pub HANDLE);

        unsafe impl Send for SendHandle {}
        unsafe impl Sync for SendHandle {}

        pub const STDOUT_NO: u32 = STD_OUTPUT_HANDLE;
        pub const STDERR_NO: u32 = STD_ERROR_HANDLE;

        pub unsafe fn get_std_handle(handle_id: u32) -> io::Result<SendHandle> {
            let handle = GetStdHandle(handle_id);
            if handle == INVALID_HANDLE_VALUE {
                Err(io::Error::last_os_error())
            } else {
                Ok(SendHandle(handle))
            }
        }

        pub unsafe fn close(handle: SendHandle) {
            CloseHandle(handle.0);
        }

        pub unsafe fn create_pipe() -> io::Result<(SendHandle, SendHandle)> {
            let mut read_handle: HANDLE = std::ptr::null_mut();
            let mut write_handle: HANDLE = std::ptr::null_mut();
            if CreatePipe(&mut read_handle, &mut write_handle, std::ptr::null(), 0) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok((SendHandle(read_handle), SendHandle(write_handle)))
            }
        }

        pub unsafe fn redirect(new_handle: SendHandle, std_handle_id: u32) -> io::Result<()> {
            if SetStdHandle(std_handle_id, new_handle.0) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        /// ハンドルを複製（リーダー用）
        pub unsafe fn dup_for_reader(handle: SendHandle) -> Option<SendHandle> {
            let mut dup_handle: HANDLE = std::ptr::null_mut();
            if DuplicateHandle(
                GetCurrentProcess(),
                handle.0,
                GetCurrentProcess(),
                &mut dup_handle,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ) == 0
            {
                None
            } else {
                Some(SendHandle(dup_handle))
            }
        }

        /// ハンドルからFileリーダーを作成
        pub unsafe fn create_reader(handle: SendHandle) -> std::fs::File {
            use std::os::windows::io::FromRawHandle;
            std::fs::File::from_raw_handle(handle.0 as _)
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
                                msg_type: MSG_TYPE_EVENT.to_string(),
                                event: EVENT_OUTPUT.to_string(),
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
