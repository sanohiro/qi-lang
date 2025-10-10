//! ファイルI/O関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::fs;
use std::io::Write;

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
        _ => Err("write-file: requires two strings (path, content)".to_string()),
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
        _ => Err("append-file: requires two strings (path, content)".to_string()),
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
        return Err("read-lines requires 1 argument".to_string());
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
        _ => Err("read-lines: argument must be a string (path)".to_string()),
    }
}

/// file-exists? - ファイルの存在を確認
pub fn native_file_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("file-exists? requires 1 argument".to_string());
    }

    match &args[0] {
        Value::String(path) => {
            Ok(Value::Bool(std::path::Path::new(path).exists()))
        }
        _ => Err("file-exists?: argument must be a string (path)".to_string()),
    }
}
