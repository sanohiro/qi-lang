//! デバッガ機能
//!
//! ブレークポイント、ステップ実行、変数検査などのデバッグ機能を提供

use parking_lot::RwLock;
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
    }

    /// ブレークポイントを削除
    pub fn remove_breakpoint(&mut self, file: &str, line: usize) {
        if let Some(lines) = self.breakpoints.get_mut(file) {
            lines.remove(&line);
        }
        self.breakpoint_details.remove(&(file.to_string(), line));
    }

    /// すべてのブレークポイントをクリア
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
        self.breakpoint_details.clear();
    }

    /// ブレークポイントが設定されているか確認
    pub fn has_breakpoint(&self, file: &str, line: usize) -> bool {
        self.breakpoints
            .get(file)
            .map(|lines| lines.contains(&line))
            .unwrap_or(false)
    }

    /// ブレークポイントでチェック（停止すべきか判定）
    pub fn check_breakpoint(&mut self, file: &str, line: usize) -> bool {
        if !self.enabled {
            return false;
        }

        match self.state {
            DebuggerState::Running => {
                // ブレークポイントがあれば停止
                if self.has_breakpoint(file, line) {
                    self.pause();
                    return true;
                }
            }
            DebuggerState::StepOver | DebuggerState::StepIn => {
                // ステップ実行中は毎行で停止
                self.pause();
                return true;
            }
            DebuggerState::StepOut => {
                // ステップアウト：目標深さに達したら停止
                if let Some(target) = self.step_out_target {
                    if self.current_depth <= target {
                        self.pause();
                        self.step_out_target = None;
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
        self.state = DebuggerState::Running;
    }

    /// ステップオーバー
    pub fn step_over(&mut self) {
        self.state = DebuggerState::StepOver;
    }

    /// ステップイン
    pub fn step_in(&mut self) {
        self.state = DebuggerState::StepIn;
    }

    /// ステップアウト
    pub fn step_out(&mut self) {
        self.state = DebuggerState::StepOut;
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
    *GLOBAL_DEBUGGER.write() = if enabled {
        Some(Debugger::new(true))
    } else {
        None
    };
}
