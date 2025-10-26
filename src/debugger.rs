//! デバッガ機能
//!
//! ブレークポイント、ステップ実行、変数検査などのデバッグ機能を提供

use parking_lot::{Condvar, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// デバッガの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    Running,  // 通常実行中
    Paused,   // ブレークポイントで停止中
    StepOver, // ステップオーバー実行中
    StepIn,   // ステップイン実行中
    StepOut,  // ステップアウト実行中
}

/// コールフレーム（スタックトレース用）
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// ブレークポイント
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub file: String,
    pub line: usize,
    pub condition: Option<String>, // 条件付きブレークポイント
    pub enabled: bool,
}

/// デバッガ
pub struct Debugger {
    /// デバッガが有効かどうか
    enabled: bool,

    /// 現在の状態
    state: DebuggerState,

    /// ブレークポイント（ファイル名 -> 行番号のセット）
    breakpoints: HashMap<String, HashSet<usize>>,

    /// ブレークポイント詳細
    breakpoint_details: HashMap<(String, usize), Breakpoint>,

    /// トレース機能が有効か
    trace_enabled: bool,

    /// コールスタック
    call_stack: Vec<CallFrame>,

    /// 現在のスタックの深さ（ステップアウト用）
    current_depth: usize,

    /// ステップアウトの目標深さ
    step_out_target: Option<usize>,

    /// 一時停止用の Condvar（Mutex と Condvar のペア）
    pause_condvar: Arc<(Mutex<()>, Condvar)>,

    /// 停止イベントが送信待ちか（ファイル名、行番号、列番号、理由）
    stopped_event_pending: Option<(String, usize, usize, String)>,

    /// 最後にヒットした位置（ファイル名、行番号、列番号） - 連続ヒット防止用
    /// 式レベルのステップ実行のため、列番号も追跡する
    last_hit_location: Option<(String, usize, usize)>,

    /// 停止時の環境（変数表示・ウォッチ式評価用）
    stopped_env: Option<Arc<RwLock<crate::value::Env>>>,
}

impl Debugger {
    /// 新しいデバッガを作成
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            state: DebuggerState::Running,
            breakpoints: HashMap::new(),
            breakpoint_details: HashMap::new(),
            trace_enabled: false,
            call_stack: Vec::new(),
            current_depth: 0,
            step_out_target: None,
            pause_condvar: Arc::new((Mutex::new(()), Condvar::new())),
            stopped_event_pending: None,
            last_hit_location: None,
            stopped_env: None,
        }
    }

    /// デバッガが有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 現在の状態を取得
    pub fn state(&self) -> DebuggerState {
        self.state
    }

    /// トレースを有効化
    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
    }

    /// トレースを無効化
    pub fn disable_trace(&mut self) {
        self.trace_enabled = false;
    }

    /// トレース出力
    pub fn trace(&self, file: &str, line: usize, expr: &str) {
        if self.trace_enabled {
            let indent = "  ".repeat(self.current_depth);
            eprintln!("[TRACE] {}{}:{} {}", indent, file, line, expr);
        }
    }

    /// ブレークポイントを追加
    pub fn add_breakpoint(&mut self, file: &str, line: usize) {
        self.breakpoints
            .entry(file.to_string())
            .or_default()
            .insert(line);

        self.breakpoint_details.insert(
            (file.to_string(), line),
            Breakpoint {
                file: file.to_string(),
                line,
                condition: None,
                enabled: true,
            },
        );

        // 追加後の確認
        let all_bps: Vec<String> = self
            .breakpoints
            .iter()
            .map(|(f, lines)| format!("{}:{:?}", f, lines))
            .collect();

        let log_msg = format!(
            "[DEBUGGER] Breakpoint added: {}:{} | All breakpoints: {:?}\n",
            file, line, all_bps
        );
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();
    }

    /// ブレークポイントを削除
    pub fn remove_breakpoint(&mut self, file: &str, line: usize) {
        if let Some(lines) = self.breakpoints.get_mut(file) {
            lines.remove(&line);
        }
        self.breakpoint_details.remove(&(file.to_string(), line));
    }

    /// 特定ファイルのブレークポイントをクリア
    pub fn clear_breakpoints_for_file(&mut self, file: &str) {
        if let Some(lines) = self.breakpoints.remove(file) {
            // breakpoint_detailsからも削除
            for line in lines {
                self.breakpoint_details.remove(&(file.to_string(), line));
            }
        }
    }

    /// すべてのブレークポイントをクリア
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
        self.breakpoint_details.clear();
    }

    /// ブレークポイントが設定されているか確認
    pub fn has_breakpoint(&self, file: &str, line: usize) -> bool {
        let file_entry = self.breakpoints.get(file);
        let result = file_entry
            .map(|lines| lines.contains(&line))
            .unwrap_or(false);

        // デバッグログ：ファイルキーとブレークポイント一覧
        if !result {
            let all_keys: Vec<String> = self.breakpoints.keys().cloned().collect();
            let log_msg = format!(
                "[DEBUGGER] has_breakpoint({}, {}) = false | All keys: {:?} | Lines for this file: {:?}\n",
                file, line, all_keys, file_entry
            );
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/qi-dap.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                .ok();
        }

        result
    }

    /// ブレークポイントでチェック（停止すべきか判定）
    /// 式レベルのステップ実行のため、列番号も受け取る
    pub fn check_breakpoint(
        &mut self,
        file: &str,
        line: usize,
        column: usize,
        env: Option<Arc<RwLock<crate::value::Env>>>,
    ) -> bool {
        if !self.enabled {
            let log_msg = format!(
                "[DEBUGGER] check_breakpoint: debugger not enabled ({}:{}:{})\n",
                file, line, column
            );
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/qi-dap.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                .ok();
            return false;
        }

        match self.state {
            DebuggerState::Running => {
                // 異なる位置（行または列）に移動した場合、最後のヒット位置をクリア
                if let Some((ref last_file, last_line, last_column)) = self.last_hit_location {
                    if last_file != file || last_line != line || last_column != column {
                        let log_msg = format!(
                            "[DEBUGGER] Moved from {}:{}:{} to {}:{}:{}, clearing last hit\n",
                            last_file, last_line, last_column, file, line, column
                        );
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/qi-dap.log")
                            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                            .ok();
                        self.last_hit_location = None;
                    }
                }

                // ブレークポイントがあれば停止
                let has_bp = self.has_breakpoint(file, line);
                let log_msg = format!(
                    "[DEBUGGER] Checking {}:{} - has_breakpoint={}\n",
                    file, line, has_bp
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

                if has_bp {
                    // 最後にヒットした位置と同じであればスキップ（連続ヒット防止）
                    if let Some((ref last_file, last_line, last_column)) = self.last_hit_location {
                        if last_file == file && last_line == line && last_column == column {
                            let log_msg = format!(
                                "[DEBUGGER] SKIPPING duplicate hit at {}:{}:{}\n",
                                file, line, column
                            );
                            std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("/tmp/qi-dap.log")
                                .and_then(|mut f| {
                                    std::io::Write::write_all(&mut f, log_msg.as_bytes())
                                })
                                .ok();
                            return false;
                        }
                    }

                    let log_msg = format!("[DEBUGGER] BREAKPOINT HIT at {}:{}\n", file, line);
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

                    self.last_hit_location = Some((file.to_string(), line, column));
                    // 環境を保存
                    if let Some(env) = env {
                        self.stopped_env = Some(env);
                    }
                    self.pause();
                    self.stopped_event_pending =
                        Some((file.to_string(), line, column, "breakpoint".to_string()));
                    return true;
                }
            }
            DebuggerState::StepIn => {
                // ステップイン: どの深さでも次の異なる式（位置）で停止
                let should_stop =
                    if let Some((ref last_file, last_line, last_column)) = self.last_hit_location {
                        last_file != file || last_line != line || last_column != column
                    } else {
                        true // 最初の停止
                    };

                if should_stop {
                    let log_msg =
                        format!("[DEBUGGER] StepIn stop at {}:{}:{}\n", file, line, column);
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

                    self.last_hit_location = Some((file.to_string(), line, column));
                    // 環境を保存
                    if let Some(env) = env {
                        self.stopped_env = Some(env);
                    }
                    self.pause();
                    self.stopped_event_pending =
                        Some((file.to_string(), line, column, "step".to_string()));
                    return true;
                } else {
                    // 同じ式で複数回チェックされた - スキップ
                    return false;
                }
            }
            DebuggerState::StepOver => {
                // ステップオーバー: 現在のスタックレベル以下で次の異なる式（位置）で停止
                // step_out_targetに記憶された深さ以下に戻るまで停止しない
                let target_depth = self.step_out_target.unwrap_or(0);

                let log_msg = format!(
                    "[DEBUGGER] StepOver check at {}:{}:{} - current_depth={}, target_depth={}\n",
                    file, line, column, self.current_depth, target_depth
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/qi-dap.log")
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                    .ok();

                // 現在の深さが目標深さ以下の場合のみ停止を検討
                let should_stop = if self.current_depth <= target_depth {
                    if let Some((ref last_file, last_line, last_column)) = self.last_hit_location {
                        last_file != file || last_line != line || last_column != column
                    } else {
                        true // 最初の停止
                    }
                } else {
                    // まだ深い位置にいるので停止しない
                    false
                };

                if should_stop {
                    let log_msg =
                        format!("[DEBUGGER] StepOver stop at {}:{}:{}\n", file, line, column);
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/qi-dap.log")
                        .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
                        .ok();

                    self.last_hit_location = Some((file.to_string(), line, column));
                    // 環境を保存
                    if let Some(env) = env {
                        self.stopped_env = Some(env);
                    }
                    self.pause();
                    self.stopped_event_pending =
                        Some((file.to_string(), line, column, "step".to_string()));
                    return true;
                } else {
                    // スタックが深いか、同じ式で複数回チェックされた - スキップ
                    return false;
                }
            }
            DebuggerState::StepOut => {
                // ステップアウト：目標深さに達したら停止
                if let Some(target) = self.step_out_target {
                    if self.current_depth <= target {
                        // 環境を保存
                        if let Some(env) = env {
                            self.stopped_env = Some(env);
                        }
                        self.pause();
                        self.step_out_target = None;
                        self.stopped_event_pending =
                            Some((file.to_string(), line, column, "step".to_string()));
                        return true;
                    }
                }
            }
            DebuggerState::Paused => {
                // 停止中は何もしない
            }
        }

        false
    }

    /// 実行を一時停止
    pub fn pause(&mut self) {
        self.state = DebuggerState::Paused;
    }

    /// 実行を再開
    pub fn resume(&mut self) {
        // stopped_event_pendingをクリア（次の停止まで保持しない）
        self.stopped_event_pending = None;

        // ステップ状態の場合は状態を保持、それ以外はRunningに
        match self.state {
            DebuggerState::Paused => {
                self.state = DebuggerState::Running;
            }
            DebuggerState::StepOver | DebuggerState::StepIn | DebuggerState::StepOut => {
                // ステップ状態は保持（Pausedではなくなるだけ）
                // 実際の状態変更は各ステップメソッドで行われている
            }
            _ => {
                // 既にRunning状態など
            }
        }
        // 注: last_hit_locationはここではクリアしない
        // 同じ行の評価中に再度ヒットするのを防ぐため、
        // 異なる行に移動した時にcheck_breakpoint内でクリアする
        // 待機中のスレッドを起こす
        let (_, cvar) = &*self.pause_condvar;
        cvar.notify_one();
    }

    /// 一時停止中であれば待機
    ///
    /// この関数は Paused 状態の場合、resume() が呼ばれるまで待機します。
    /// GLOBAL_DEBUGGER のロックを保持せずに呼び出す必要があります。
    pub fn wait_if_paused(&self) {
        // 状態をチェック
        if self.state != DebuggerState::Paused {
            return;
        }

        // Condvar で待機
        let (lock, cvar) = &*self.pause_condvar;
        let mut guard = lock.lock();

        // resume() されるまで待機
        // 状態を再チェックしながら待機（spurious wakeup 対策）
        // NOTE: condvar.wait()内でself.stateが変更されるため、無限ループではない
        #[allow(clippy::while_immutable_condition)]
        while self.state == DebuggerState::Paused {
            cvar.wait(&mut guard);
        }
    }

    /// ステップオーバー
    pub fn step_over(&mut self) {
        self.state = DebuggerState::StepOver;
        // 注: last_hit_locationはクリアしない
        // 現在停止している位置が記録されているので、次のcheck_breakpoint()で
        // 異なる行に来た時だけ停止するようにする
        // 現在の深さを記憶（この深さ以下に戻ったら止まる）
        self.step_out_target = Some(self.current_depth);
    }

    /// ステップイン
    pub fn step_in(&mut self) {
        self.state = DebuggerState::StepIn;
        // 注: last_hit_locationはクリアしない
        // 現在停止している位置が記録されているので、次のcheck_breakpoint()で
        // 異なる行に来た時だけ停止するようにする
    }

    /// ステップアウト
    pub fn step_out(&mut self) {
        self.state = DebuggerState::StepOut;
        // 注: last_hit_locationはクリアしない
        // 現在停止している位置が記録されているので、次のcheck_breakpoint()で
        // 異なる行に来た時だけ停止するようにする
        self.step_out_target = Some(self.current_depth.saturating_sub(1));
    }

    /// 関数呼び出し開始
    pub fn enter_function(&mut self, name: &str, file: &str, line: usize, column: usize) {
        self.current_depth += 1;
        self.call_stack.push(CallFrame {
            function_name: name.to_string(),
            file: file.to_string(),
            line,
            column,
        });

        if self.trace_enabled {
            let indent = "  ".repeat(self.current_depth - 1);
            eprintln!("[TRACE] {}-> {} ({}:{})", indent, name, file, line);
        }
    }

    /// 関数呼び出し終了
    pub fn exit_function(&mut self) {
        if self.trace_enabled && !self.call_stack.is_empty() {
            let frame = &self.call_stack[self.call_stack.len() - 1];
            let indent = "  ".repeat(self.current_depth - 1);
            eprintln!("[TRACE] {}<- {}", indent, frame.function_name);
        }

        self.call_stack.pop();
        self.current_depth = self.current_depth.saturating_sub(1);
    }

    /// コールスタックを取得
    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// スタックトレースを文字列として取得
    pub fn stack_trace(&self) -> String {
        let mut trace = String::new();
        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            trace.push_str(&format!(
                "  #{} {} at {}:{}\n",
                i, frame.function_name, frame.file, frame.line
            ));
        }
        trace
    }

    /// 停止イベントが待機中か確認し、待機中であれば取得してクリア
    ///
    /// 戻り値: (ファイル名, 行番号, 列番号, 理由)
    pub fn take_stopped_event(&mut self) -> Option<(String, usize, usize, String)> {
        self.stopped_event_pending.take()
    }

    /// 停止イベント情報を参照のみ（クリアしない）
    pub fn get_stopped_event(&self) -> Option<&(String, usize, usize, String)> {
        self.stopped_event_pending.as_ref()
    }

    /// 停止時の環境を保存
    pub fn set_stopped_env(&mut self, env: Arc<RwLock<crate::value::Env>>) {
        self.stopped_env = Some(env);
    }

    /// 停止時の環境を取得
    pub fn get_stopped_env(&self) -> Option<Arc<RwLock<crate::value::Env>>> {
        self.stopped_env.clone()
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new(false)
    }
}

/// グローバルデバッガインスタンス（DAPサーバーから参照するため）
pub type SharedDebugger = Arc<RwLock<Option<Debugger>>>;

/// グローバルデバッガを作成
pub fn create_shared_debugger(enabled: bool) -> SharedDebugger {
    Arc::new(RwLock::new(if enabled {
        Some(Debugger::new(true))
    } else {
        None
    }))
}

/// グローバルデバッガインスタンス
use std::sync::LazyLock;
pub static GLOBAL_DEBUGGER: LazyLock<SharedDebugger> =
    LazyLock::new(|| create_shared_debugger(false));

/// グローバルデバッガを初期化
pub fn init_global_debugger(enabled: bool) {
    let mut guard = GLOBAL_DEBUGGER.write();

    // 既に初期化済みの場合は何もしない
    if guard.is_some() {
        let log_msg = "[DEBUGGER] init_global_debugger: Already initialized, skipping\n";
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();
        return;
    }

    *guard = if enabled {
        let log_msg = "[DEBUGGER] init_global_debugger: Creating new debugger instance\n";
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/qi-dap.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()))
            .ok();
        Some(Debugger::new(true))
    } else {
        None
    };
}

/// グローバルデバッガが一時停止中であれば待機
///
/// この関数は GLOBAL_DEBUGGER が Paused 状態の場合、resume() が呼ばれるまで待機します。
/// eval.rs から呼び出すための便利関数です。
pub fn wait_if_paused_global() {
    // GLOBAL_DEBUGGER から pause_condvar をクローン
    let condvar = {
        let guard = GLOBAL_DEBUGGER.read();
        if let Some(ref dbg) = *guard {
            if dbg.state != DebuggerState::Paused {
                return;
            }
            Arc::clone(&dbg.pause_condvar)
        } else {
            return;
        }
    };

    // ロックを解放した状態で待機
    let (lock, cvar) = &*condvar;
    let mut guard = lock.lock();

    // 状態を再チェックしながら待機（spurious wakeup 対策）
    loop {
        let state = {
            let g = GLOBAL_DEBUGGER.read();
            g.as_ref()
                .map(|d| d.state)
                .unwrap_or(DebuggerState::Running)
        };
        if state != DebuggerState::Paused {
            break;
        }
        cvar.wait(&mut guard);
    }
}
