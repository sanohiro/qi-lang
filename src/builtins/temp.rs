//! 一時ファイル・ディレクトリ関数
//!
//! このモジュールは `io-temp` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;
use tempfile::{NamedTempFile, TempDir};

// 自動削除される一時ファイル・ディレクトリのハンドルを保持
fn temp_files() -> &'static Mutex<HashMap<String, NamedTempFile>> {
    static TEMP_FILES: OnceLock<Mutex<HashMap<String, NamedTempFile>>> = OnceLock::new();
    TEMP_FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn temp_dirs() -> &'static Mutex<HashMap<String, TempDir>> {
    static TEMP_DIRS: OnceLock<Mutex<HashMap<String, TempDir>>> = OnceLock::new();
    TEMP_DIRS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// temp-file - 一時ファイルを作成（自動削除）
/// 引数: なし
/// 返値: 一時ファイルのパス
/// 例: (io/temp-file) => "/tmp/qi-12345.tmp"
pub fn native_temp_file(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["io/temp-file"]));
    }

    let temp_file = NamedTempFile::new()
        .map_err(|e| format!("io/temp-file: failed to create temp file: {}", e))?;

    let path = temp_file
        .path()
        .to_str()
        .ok_or("io/temp-file: invalid path")?
        .to_string();

    // ハンドルを保持して自動削除を有効にする
    temp_files().lock().insert(path.clone(), temp_file);

    Ok(Value::String(path))
}

/// temp-file-keep - 一時ファイルを作成（削除しない）
/// 引数: なし
/// 返値: 一時ファイルのパス
/// 例: (io/temp-file-keep) => "/tmp/qi-12345.tmp"
pub fn native_temp_file_keep(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["io/temp-file-keep"]));
    }

    let temp_file = NamedTempFile::new()
        .map_err(|e| format!("io/temp-file-keep: failed to create temp file: {}", e))?;

    // persist()でファイルを永続化（自動削除を無効化）
    let (_, path) = temp_file
        .keep()
        .map_err(|e| format!("io/temp-file-keep: failed to persist temp file: {}", e))?;

    let path_str = path
        .to_str()
        .ok_or("io/temp-file-keep: invalid path")?
        .to_string();

    Ok(Value::String(path_str))
}

/// temp-dir - 一時ディレクトリを作成（自動削除）
/// 引数: なし
/// 返値: 一時ディレクトリのパス
/// 例: (io/temp-dir) => "/tmp/qi-dir-12345"
pub fn native_temp_dir(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["io/temp-dir"]));
    }

    let temp_dir = TempDir::new()
        .map_err(|e| format!("io/temp-dir: failed to create temp directory: {}", e))?;

    let path = temp_dir
        .path()
        .to_str()
        .ok_or("io/temp-dir: invalid path")?
        .to_string();

    // ハンドルを保持して自動削除を有効にする
    temp_dirs().lock().insert(path.clone(), temp_dir);

    Ok(Value::String(path))
}

/// temp-dir-keep - 一時ディレクトリを作成（削除しない）
/// 引数: なし
/// 返値: 一時ディレクトリのパス
/// 例: (io/temp-dir-keep) => "/tmp/qi-dir-12345"
pub fn native_temp_dir_keep(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["io/temp-dir-keep"]));
    }

    let temp_dir = TempDir::new()
        .map_err(|e| format!("io/temp-dir-keep: failed to create temp directory: {}", e))?;

    // into_path()でディレクトリを永続化（自動削除を無効化）
    #[allow(deprecated)]
    let path = temp_dir.into_path();

    let path_str = path
        .to_str()
        .ok_or("io/temp-dir-keep: invalid path")?
        .to_string();

    Ok(Value::String(path_str))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
pub const FUNCTIONS: super::NativeFunctions = &[
    ("io/temp-file", native_temp_file),
    ("io/temp-file-keep", native_temp_file_keep),
    ("io/temp-dir", native_temp_dir),
    ("io/temp-dir-keep", native_temp_dir_keep),
];
