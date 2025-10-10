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
