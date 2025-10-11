//! ファイルI/O関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

/// read-file - ファイルを読み込む
pub fn native_read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["read-file"]));
    }

    match &args[0] {
        Value::String(path) => {
            match fs::read_to_string(path) {
                Ok(content) => Ok(Value::String(content)),
                Err(e) => Err(format!("read-file: failed to read {}: {}", path, e)),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["read-file", "strings"])),
    }
}

/// write-file - ファイルに書き込む（上書き）
pub fn native_write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["write-file"]));
    }

    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            match fs::write(path, content) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("write-file: failed to write {}: {}", path, e)),
            }
        }
        _ => Err(fmt_msg(MsgKey::NeedNArgsDesc, &["write-file", "2", "(path: string, content: string)"])),
    }
}

/// append-file - ファイルに追記
pub fn native_append_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["append-file"]));
    }

    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            match fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Ok(Value::Nil),
                        Err(e) => Err(format!("append-file: failed to write {}: {}", path, e)),
                    }
                }
                Err(e) => Err(format!("append-file: failed to open {}: {}", path, e)),
            }
        }
        _ => Err(fmt_msg(MsgKey::NeedNArgsDesc, &["append-file", "2", "(path: string, content: string)"])),
    }
}

/// println - 改行付きで出力
pub fn native_println(args: &[Value]) -> Result<Value, String> {
    let output = if args.is_empty() {
        String::new()
    } else {
        args.iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => format!("{:?}", v),
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!("{}", output);
    Ok(Value::Nil)
}

/// read-lines - ファイルを行ごとに読み込み
pub fn native_read_lines(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["read-lines"]));
    }

    match &args[0] {
        Value::String(path) => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let lines: Vec<Value> = content
                        .lines()
                        .map(|line| Value::String(line.to_string()))
                        .collect();
                    Ok(Value::List(lines))
                }
                Err(e) => Err(format!("read-lines: failed to read {}: {}", path, e)),
            }
        }
        _ => Err(fmt_msg(MsgKey::MustBeString, &["read-lines", "argument"])),
    }
}

/// file-exists? - ファイルの存在を確認
pub fn native_file_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["file-exists?"]));
    }

    match &args[0] {
        Value::String(path) => {
            Ok(Value::Bool(std::path::Path::new(path).exists()))
        }
        _ => Err(fmt_msg(MsgKey::MustBeString, &["file-exists?", "argument"])),
    }
}

/// file-stream - ファイルを遅延読み込み（ストリーミング）
/// 引数: (file-stream "path") - テキストモード（行ごと）
///      (file-stream "path" :bytes) - バイナリモード（バイトベクタごと）
pub fn native_file_stream(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["file-stream"]));
    }

    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["file-stream", "a string"])),
    };

    // 第2引数でバイナリモード判定
    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if k == "bytes");

    if is_bytes {
        create_file_byte_stream(&path)
    } else {
        create_file_line_stream(&path)
    }
}

/// ファイルを行ごとに読み込むストリーム（テキストモード）
fn create_file_line_stream(path: &str) -> Result<Value, String> {
    let file = File::open(path)
        .map_err(|e| format!("file-stream: failed to open '{}': {}", path, e))?;

    let reader = Arc::new(RwLock::new(BufReader::new(file)));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut reader = reader.write();
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => None,  // EOF
                Ok(_) => {
                    // 末尾の改行を削除
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Some(Value::String(line))
                }
                Err(_) => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// ファイルをバイトごとに読み込むストリーム（バイナリモード）
fn create_file_byte_stream(path: &str) -> Result<Value, String> {
    use std::io::Read;

    let file = File::open(path)
        .map_err(|e| format!("file-stream: failed to open '{}': {}", path, e))?;

    let reader = Arc::new(RwLock::new(BufReader::new(file)));
    const CHUNK_SIZE: usize = 4096; // 4KB chunks

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut reader = reader.write();
            let mut buffer = vec![0u8; CHUNK_SIZE];
            match reader.read(&mut buffer) {
                Ok(0) => None,  // EOF
                Ok(n) => {
                    buffer.truncate(n);
                    // バイト配列をIntegerのVectorに変換
                    let bytes: Vec<Value> = buffer.iter().map(|&b| Value::Integer(b as i64)).collect();
                    Some(Value::Vector(bytes))
                }
                Err(_) => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}
