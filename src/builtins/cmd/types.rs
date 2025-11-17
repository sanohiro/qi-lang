//! コマンド実行 - 型定義

/// プロセスストリーム（双方向通信用）
pub(super) type ProcessStreams = (
    Option<std::process::ChildStdin>,
    Option<std::io::BufReader<std::process::ChildStdout>>,
    Option<std::io::BufReader<std::process::ChildStderr>>,
    std::process::Child,
);
