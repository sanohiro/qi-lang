//! ファイルI/O関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use encoding_rs::{Encoding, UTF_8, SHIFT_JIS, EUC_JP, ISO_2022_JP};

// ============================================
// エンコーディング処理ユーティリティ
// ============================================

/// エンコーディングを解決
fn resolve_encoding(keyword: &str) -> Result<&'static Encoding, String> {
    match keyword {
        "utf-8" | "utf8" => Ok(UTF_8),
        "utf-8-bom" | "utf8-bom" => Ok(UTF_8), // BOMは別途処理
        "sjis" | "shift-jis" | "shift_jis" => Ok(SHIFT_JIS),
        "euc-jp" | "euc_jp" | "eucjp" => Ok(EUC_JP),
        "iso-2022-jp" | "iso2022jp" => Ok(ISO_2022_JP),
        _ => Err(format!("Unsupported encoding: {}", keyword)),
    }
}

/// BOMをチェックして除去
fn strip_bom(bytes: &[u8]) -> (&[u8], bool) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (&bytes[3..], true)
    } else {
        (bytes, false)
    }
}

/// バイト列をUTF-8文字列にデコード
fn decode_bytes(bytes: &[u8], encoding: &'static Encoding, path: &str) -> Result<String, String> {
    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        Err(format!("{}: failed to decode as {} (invalid byte sequence)", path, encoding.name()))
    } else {
        Ok(decoded.into_owned())
    }
}

/// 自動エンコーディング検出（ベストエフォート）
fn auto_detect_encoding(bytes: &[u8], path: &str) -> Result<String, String> {
    // BOMチェック
    let (bytes_without_bom, has_bom) = strip_bom(bytes);
    if has_bom {
        return decode_bytes(bytes_without_bom, UTF_8, path);
    }

    // UTF-8として試行
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Ok(s);
    }

    // Shift_JISとして試行
    if let Ok(s) = decode_bytes(bytes, SHIFT_JIS, path) {
        return Ok(s);
    }

    // EUC-JPとして試行
    if let Ok(s) = decode_bytes(bytes, EUC_JP, path) {
        return Ok(s);
    }

    // ISO-2022-JPとして試行
    if let Ok(s) = decode_bytes(bytes, ISO_2022_JP, path) {
        return Ok(s);
    }

    Err(format!("{}: could not detect encoding (tried UTF-8, Shift_JIS, EUC-JP, ISO-2022-JP)", path))
}

/// 文字列をバイト列にエンコード
fn encode_string(content: &str, encoding: &'static Encoding, add_bom: bool) -> Vec<u8> {
    let (encoded, _, _) = encoding.encode(content);
    let mut result = Vec::new();

    if add_bom && encoding == UTF_8 {
        result.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }

    result.extend_from_slice(&encoded);
    result
}

/// キーワード引数からオプションを抽出
fn parse_keyword_args(args: &[Value], start_idx: usize) -> Result<std::collections::HashMap<String, Value>, String> {
    let mut opts = std::collections::HashMap::new();
    let mut i = start_idx;

    while i < args.len() {
        match &args[i] {
            Value::Keyword(key) => {
                if i + 1 >= args.len() {
                    return Err(format!("Keyword :{} requires a value", key));
                }
                opts.insert(key.clone(), args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(format!("Expected keyword argument, got {:?}", args[i]));
            }
        }
    }

    Ok(opts)
}

/// read-file - ファイルを読み込む
/// 引数: (path) または (path :encoding :sjis)
/// サポートされるオプション:
///   :encoding :utf-8 (デフォルト、BOM自動除去)
///   :encoding :utf-8-bom (BOM付きUTF-8)
///   :encoding :sjis (Shift_JIS)
///   :encoding :euc-jp (EUC-JP)
///   :encoding :auto (自動検出)
pub fn native_read_file(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["read-file"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["read-file (1st arg)", "string"])),
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // エンコーディングオプションを取得
    let encoding_keyword = opts.get("encoding")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None
        })
        .unwrap_or("utf-8");

    // ファイルをバイト列として読み込み
    let bytes = fs::read(path)
        .map_err(|e| format!("{}: {}", path, e))?;

    // エンコーディングに応じてデコード
    let content = if encoding_keyword == "auto" {
        auto_detect_encoding(&bytes, path)?
    } else {
        let (bytes_to_decode, _has_bom) = strip_bom(&bytes);

        if encoding_keyword == "utf-8" || encoding_keyword == "utf-8-bom" {
            // UTF-8の場合はBOMを自動除去
            String::from_utf8(bytes_to_decode.to_vec())
                .map_err(|_| format!("{}: failed to decode as UTF-8 (invalid byte sequence)", path))?
        } else {
            let encoding = resolve_encoding(encoding_keyword)?;
            decode_bytes(bytes_to_decode, encoding, path)?
        }
    };

    Ok(Value::String(content))
}

/// write-file - ファイルに書き込む（上書き）
/// 引数: (content, path) または (content, path :encoding :sjis :if-exists :error :create-dirs true)
/// パイプライン対応: (content |> (io/write-file "output.txt"))
///
/// サポートされるオプション:
///   :encoding :utf-8 (デフォルト)
///   :encoding :utf-8-bom (BOM付きUTF-8)
///   :encoding :sjis (Shift_JIS)
///   :encoding :euc-jp (EUC-JP)
///
///   :if-exists :overwrite (デフォルト、上書き)
///   :if-exists :error (存在したらエラー)
///   :if-exists :skip (存在したらスキップ)
///   :if-exists :append (追記)
///
///   :create-dirs true (ディレクトリを自動作成、デフォルトfalse)
pub fn native_write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["write-file"]));
    }

    let content = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["write-file (1st arg - content)", "string"])),
    };

    let path = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["write-file (2nd arg - path)", "string"])),
    };

    // キーワード引数を解析
    let opts = if args.len() > 2 {
        parse_keyword_args(args, 2)?
    } else {
        std::collections::HashMap::new()
    };

    // エンコーディングオプション
    let encoding_keyword = opts.get("encoding")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None
        })
        .unwrap_or("utf-8");

    // if-existsオプション
    let if_exists = opts.get("if-exists")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None
        })
        .unwrap_or("overwrite");

    // create-dirsオプション
    let create_dirs = opts.get("create-dirs")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None
        })
        .unwrap_or(false);

    let path_obj = Path::new(path);

    // ディレクトリ自動作成
    if create_dirs {
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: failed to create directory: {}", parent.display(), e))?;
            }
        }
    }

    // ファイル存在チェック
    if path_obj.exists() {
        match if_exists {
            "error" => {
                return Err(format!("{}: file already exists", path));
            }
            "skip" => {
                return Ok(Value::Nil);
            }
            "append" => {
                // 追記モード
                let add_bom = encoding_keyword == "utf-8-bom";
                let encoding = resolve_encoding(encoding_keyword)?;
                let bytes = encode_string(content, encoding, add_bom);

                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .map_err(|e| format!("{}: failed to open for append: {}", path, e))?;

                file.write_all(&bytes)
                    .map_err(|e| format!("{}: failed to append: {}", path, e))?;

                return Ok(Value::Nil);
            }
            "overwrite" => {
                // 上書き（デフォルト）
            }
            _ => {
                return Err(format!("Invalid :if-exists option: {}", if_exists));
            }
        }
    }

    // エンコードして書き込み
    let add_bom = encoding_keyword == "utf-8-bom";
    let encoding = resolve_encoding(encoding_keyword)?;
    let bytes = encode_string(content, encoding, add_bom);

    fs::write(path, bytes)
        .map_err(|e| format!("{}: failed to write: {}", path, e))?;

    Ok(Value::Nil)
}

/// append-file - ファイルに追記
/// 引数: (content, path) - パイプライン対応
/// 使い方: (content |> (io/append-file "log.txt"))
pub fn native_append_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["append-file"]));
    }

    match (&args[0], &args[1]) {
        (Value::String(content), Value::String(path)) => {
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
        _ => Err(fmt_msg(MsgKey::NeedNArgsDesc, &["append-file", "2", "(content: string, path: string)"])),
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

/// write-stream - ストリームをファイルに書き込み
/// 引数: (stream, path) - パイプライン対応
/// 使い方: (stream |> (io/write-stream "output.txt"))
/// ストリームの各要素を文字列に変換して改行付きで書き込む
pub fn native_write_stream(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["write-stream"]));
    }

    let stream = match &args[0] {
        Value::Stream(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["write-stream (1st arg)", "a stream"])),
    };

    let path = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["write-stream (2nd arg)", "string"])),
    };

    // ファイルを開く
    let mut file = fs::File::create(path)
        .map_err(|e| format!("write-stream: failed to create {}: {}", path, e))?;

    // ストリームの各要素をファイルに書き込み
    let mut count = 0;
    loop {
        let next_val = {
            let s = stream.read();
            (s.next_fn)()
        };

        match next_val {
            Some(val) => {
                let line = match &val {
                    Value::String(s) => s.clone(),
                    Value::Integer(n) => n.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Nil => String::from("nil"),
                    _ => format!("{:?}", val),
                };

                writeln!(file, "{}", line)
                    .map_err(|e| format!("write-stream: failed to write to {}: {}", path, e))?;
                count += 1;
            }
            None => break,
        }
    }

    Ok(Value::Integer(count))
}
