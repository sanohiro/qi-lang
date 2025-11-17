//! コマンド実行関数
//!
//! Unixコマンドをパイプラインで実行できる関数群。
//! データの流れとしてコマンドを扱い、|> と統合される。
//!
//! ## セキュリティ警告
//!
//! シェル経由でコマンドを実行する関数（文字列としてコマンドを渡す場合）は、
//! コマンドインジェクション攻撃のリスクがあります。
//!
//! **安全な使い方**:
//! - ハードコードされたコマンド文字列のみ使用
//! - ユーザー入力を含む場合は配列形式で渡す（`["cmd" "arg1" "arg2"]`）
//!
//! **危険な例**:
//! ```qi
//! ;; 危険！ユーザー入力を文字列連結している
//! (def user-input (http/get-param req "file"))
//! (cmd/exec (str "cat " user-input))  ;; コマンドインジェクションの危険
//! ```
//!
//! **安全な例**:
//! ```qi
//! ;; 安全：ハードコードされたコマンド
//! (cmd/exec "ls -la")
//!
//! ;; 安全：配列形式で渡す（シェル経由しない）
//! (def user-input (http/get-param req "file"))
//! (cmd/exec ["cat" user-input])  ;; シェルメタ文字がエスケープされる
//! ```
//!
//! このモジュールは `cmd-exec` feature でコンパイルされます。

pub mod exec;
pub mod helpers;
pub mod interactive;
pub mod pipe;
pub mod stream;
pub mod types;

pub use exec::*;
pub use interactive::*;
pub use pipe::*;
pub use stream::*;

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category cmd
/// @qi-doc:functions exec, exec!, pipe, pipe!, lines, stream-lines, stream-bytes, interactive, write, read-line, wait
pub const FUNCTIONS: super::NativeFunctions = &[
    ("cmd/exec", native_exec),
    ("cmd/exec!", native_exec_bang),
    ("cmd/pipe", native_pipe),
    ("cmd/pipe!", native_pipe_bang),
    ("cmd/lines", native_lines),
    ("cmd/stream-lines", native_stream_lines),
    ("cmd/stream-bytes", native_stream_bytes),
    ("cmd/interactive", native_interactive),
    ("cmd/write", native_proc_write),
    ("cmd/read-line", native_proc_read_line),
    ("cmd/wait", native_proc_wait),
];
