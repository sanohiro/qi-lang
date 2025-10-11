//! パス操作関数

use crate::value::Value;
use std::path::{Path, PathBuf, Component};
use std::env;

/// join - パスを結合
/// 引数: (parts...) - 結合するパス要素のリスト
/// 例: (path/join "dir" "subdir" "file.txt") => "dir/subdir/file.txt"
pub fn native_path_join(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("path/join: at least 1 argument required".to_string());
    }

    let mut path = PathBuf::new();
    for arg in args {
        match arg {
            Value::String(s) => path.push(s),
            _ => return Err(format!("path/join: all arguments must be strings, got {:?}", arg)),
        }
    }

    Ok(Value::String(path.to_string_lossy().to_string()))
}

/// basename - ファイル名部分を取得
/// 引数: (path) - パス文字列
/// 例: (path/basename "/path/to/file.txt") => "file.txt"
pub fn native_path_basename(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/basename: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            match path.file_name() {
                Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Err("path/basename: argument must be a string".to_string()),
    }
}

/// dirname - ディレクトリ部分を取得
/// 引数: (path) - パス文字列
/// 例: (path/dirname "/path/to/file.txt") => "/path/to"
pub fn native_path_dirname(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/dirname: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            match path.parent() {
                Some(parent) => Ok(Value::String(parent.to_string_lossy().to_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Err("path/dirname: argument must be a string".to_string()),
    }
}

/// extension - 拡張子を取得
/// 引数: (path) - パス文字列
/// 例: (path/extension "file.txt") => "txt"
pub fn native_path_extension(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/extension: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            match path.extension() {
                Some(ext) => Ok(Value::String(ext.to_string_lossy().to_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Err("path/extension: argument must be a string".to_string()),
    }
}

/// stem - 拡張子なしのファイル名を取得
/// 引数: (path) - パス文字列
/// 例: (path/stem "file.txt") => "file"
pub fn native_path_stem(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/stem: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            match path.file_stem() {
                Some(stem) => Ok(Value::String(stem.to_string_lossy().to_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Err("path/stem: argument must be a string".to_string()),
    }
}

/// absolute - 絶対パスに変換
/// 引数: (path) - パス文字列
/// 例: (path/absolute "relative/path") => "/current/dir/relative/path"
pub fn native_path_absolute(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/absolute: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                env::current_dir()
                    .map_err(|e| format!("path/absolute: failed to get current directory: {}", e))?
                    .join(path)
            };

            Ok(Value::String(abs_path.to_string_lossy().to_string()))
        }
        _ => Err("path/absolute: argument must be a string".to_string()),
    }
}

/// normalize - パスを正規化（. や .. を解決）
/// 引数: (path) - パス文字列
/// 例: (path/normalize "a/./b/../c") => "a/c"
pub fn native_path_normalize(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/normalize: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            let mut normalized = PathBuf::new();

            for component in path.components() {
                match component {
                    Component::CurDir => {
                        // . は無視
                    }
                    Component::ParentDir => {
                        // .. は1つ上のディレクトリに戻る
                        normalized.pop();
                    }
                    _ => {
                        normalized.push(component);
                    }
                }
            }

            Ok(Value::String(normalized.to_string_lossy().to_string()))
        }
        _ => Err("path/normalize: argument must be a string".to_string()),
    }
}

/// is-absolute? - 絶対パスかどうか判定
/// 引数: (path) - パス文字列
/// 例: (path/is-absolute? "/usr/bin") => true
pub fn native_path_is_absolute(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/is-absolute?: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            Ok(Value::Bool(path.is_absolute()))
        }
        _ => Err("path/is-absolute?: argument must be a string".to_string()),
    }
}

/// is-relative? - 相対パスかどうか判定
/// 引数: (path) - パス文字列
/// 例: (path/is-relative? "data/file.txt") => true
pub fn native_path_is_relative(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("path/is-relative?: exactly 1 argument required".to_string());
    }

    match &args[0] {
        Value::String(s) => {
            let path = Path::new(s);
            Ok(Value::Bool(path.is_relative()))
        }
        _ => Err("path/is-relative?: argument must be a string".to_string()),
    }
}
