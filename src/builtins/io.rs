//! ファイルI/O関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use encoding_rs::{
    Encoding, UTF_8, UTF_16LE, UTF_16BE,
    SHIFT_JIS, EUC_JP, ISO_2022_JP,  // 日本語
    GBK, GB18030, BIG5,               // 中国語（簡体字・繁体字）
    EUC_KR,                           // 韓国語
    WINDOWS_1252, WINDOWS_1251,       // 欧州（西欧・ロシア）
};

// ============================================
// エンコーディング処理ユーティリティ
// ============================================

/// エンコーディングを解決
fn resolve_encoding(keyword: &str) -> Result<&'static Encoding, String> {
    match keyword {
        // Unicode
        "utf-8" | "utf8" => Ok(UTF_8),
        "utf-8-bom" | "utf8-bom" => Ok(UTF_8), // BOMは別途処理
        "utf-16le" | "utf16le" => Ok(UTF_16LE),
        "utf-16be" | "utf16be" => Ok(UTF_16BE),

        // 日本語
        "sjis" | "shift-jis" | "shift_jis" => Ok(SHIFT_JIS),
        "euc-jp" | "euc_jp" | "eucjp" => Ok(EUC_JP),
        "iso-2022-jp" | "iso2022jp" => Ok(ISO_2022_JP),

        // 中国語
        "gbk" => Ok(GBK),
        "gb18030" => Ok(GB18030),
        "big5" => Ok(BIG5),

        // 韓国語
        "euc-kr" | "euc_kr" | "euckr" => Ok(EUC_KR),

        // 欧州
        "windows-1252" | "cp1252" | "latin1" => Ok(WINDOWS_1252),
        "windows-1251" | "cp1251" => Ok(WINDOWS_1251),

        _ => Err(fmt_msg(MsgKey::UnsupportedEncoding, &[keyword])),
    }
}

/// BOMをチェックして除去（エンコーディングも返す）
fn strip_bom(bytes: &[u8]) -> (&[u8], Option<&'static Encoding>) {
    // UTF-8 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (&bytes[3..], Some(UTF_8));
    }
    // UTF-16LE BOM
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return (&bytes[2..], Some(UTF_16LE));
    }
    // UTF-16BE BOM
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return (&bytes[2..], Some(UTF_16BE));
    }
    (bytes, None)
}

/// バイト列をUTF-8文字列にデコード
fn decode_bytes(bytes: &[u8], encoding: &'static Encoding, path: &str) -> Result<String, String> {
    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        Err(fmt_msg(MsgKey::IoFailedToDecodeAs, &[path, encoding.name()]))
    } else {
        Ok(decoded.into_owned())
    }
}

/// 自動エンコーディング検出（ベストエフォート）
fn auto_detect_encoding(bytes: &[u8], path: &str) -> Result<String, String> {
    // BOMチェック（UTF-8/UTF-16LE/UTF-16BE）
    let (bytes_without_bom, detected_encoding) = strip_bom(bytes);
    if let Some(encoding) = detected_encoding {
        return decode_bytes(bytes_without_bom, encoding, path);
    }

    // UTF-8として試行（BOMなし）
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Ok(s);
    }

    // 日本語エンコーディング
    let japanese_encodings = [SHIFT_JIS, EUC_JP, ISO_2022_JP];
    for encoding in &japanese_encodings {
        if let Ok(s) = decode_bytes(bytes, encoding, path) {
            return Ok(s);
        }
    }

    // 中国語エンコーディング（簡体字・繁体字）
    let chinese_encodings = [GBK, GB18030, BIG5];
    for encoding in &chinese_encodings {
        if let Ok(s) = decode_bytes(bytes, encoding, path) {
            return Ok(s);
        }
    }

    // 韓国語エンコーディング
    if let Ok(s) = decode_bytes(bytes, EUC_KR, path) {
        return Ok(s);
    }

    // 欧州エンコーディング
    let european_encodings = [WINDOWS_1252, WINDOWS_1251];
    for encoding in &european_encodings {
        if let Ok(s) = decode_bytes(bytes, encoding, path) {
            return Ok(s);
        }
    }

    Err(fmt_msg(MsgKey::IoCouldNotDetectEncoding, &[path]))
}

/// 文字列をバイト列にエンコード
fn encode_string(content: &str, encoding: &'static Encoding, add_bom: bool) -> Vec<u8> {
    let mut result = Vec::new();

    // UTF-16は手動でエンコード（encoding_rsがエンコーダーをサポートしていないため）
    // UTF-16はデフォルトでBOM付き（Excel互換）
    if encoding == UTF_16LE {
        // UTF-16LEはデフォルトでBOM付き
        result.extend_from_slice(&[0xFF, 0xFE]); // UTF-16LE BOM
        for code_unit in content.encode_utf16() {
            result.extend_from_slice(&code_unit.to_le_bytes());
        }
        return result;
    }

    if encoding == UTF_16BE {
        // UTF-16BEはデフォルトでBOM付き
        result.extend_from_slice(&[0xFE, 0xFF]); // UTF-16BE BOM
        for code_unit in content.encode_utf16() {
            result.extend_from_slice(&code_unit.to_be_bytes());
        }
        return result;
    }

    // その他のエンコーディング
    if add_bom && encoding == UTF_8 {
        result.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }

    let (encoded, _, _) = encoding.encode(content);
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
                    return Err(fmt_msg(MsgKey::KeywordRequiresValue, &[key]));
                }
                opts.insert(key.clone(), args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(fmt_msg(MsgKey::ExpectedKeywordArg, &[&format!("{:?}", args[i])]));
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
        .map_err(|e| fmt_msg(MsgKey::IoFileError, &[path, &e.to_string()]))?;

    // エンコーディングに応じてデコード
    let content = if encoding_keyword == "auto" {
        auto_detect_encoding(&bytes, path)?
    } else {
        let (bytes_to_decode, _detected_encoding) = strip_bom(&bytes);

        if encoding_keyword == "utf-8" || encoding_keyword == "utf-8-bom" {
            // UTF-8の場合はBOMを自動除去
            String::from_utf8(bytes_to_decode.to_vec())
                .map_err(|_| fmt_msg(MsgKey::IoFailedToDecodeUtf8, &[path]))?
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
                    .map_err(|e| fmt_msg(MsgKey::IoFailedToCreateDir, &[&parent.display().to_string(), &e.to_string()]))?;
            }
        }
    }

    // ファイル存在チェック
    if path_obj.exists() {
        match if_exists {
            "error" => {
                return Err(fmt_msg(MsgKey::FileAlreadyExists, &[path]));
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
                    .map_err(|e| fmt_msg(MsgKey::IoFailedToOpenForAppend, &[path, &e.to_string()]))?;

                file.write_all(&bytes)
                    .map_err(|e| fmt_msg(MsgKey::IoFailedToAppend, &[path, &e.to_string()]))?;

                return Ok(Value::Nil);
            }
            "overwrite" => {
                // 上書き（デフォルト）
            }
            _ => {
                return Err(fmt_msg(MsgKey::InvalidIfExistsOption, &[if_exists]));
            }
        }
    }

    // エンコードして書き込み
    let add_bom = encoding_keyword == "utf-8-bom";
    let encoding = resolve_encoding(encoding_keyword)?;
    let bytes = encode_string(content, encoding, add_bom);

    fs::write(path, bytes)
        .map_err(|e| fmt_msg(MsgKey::IoFailedToWrite, &[path, &e.to_string()]))?;

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
                        Err(e) => Err(fmt_msg(MsgKey::IoAppendFileFailedToWrite, &[path, &e.to_string()])),
                    }
                }
                Err(e) => Err(fmt_msg(MsgKey::IoAppendFileFailedToOpen, &[path, &e.to_string()])),
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
                Err(e) => Err(fmt_msg(MsgKey::IoReadLinesFailedToRead, &[path, &e.to_string()])),
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
        .map_err(|e| fmt_msg(MsgKey::FileStreamFailedToOpen, &[path, &e.to_string()]))?;

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
        .map_err(|e| fmt_msg(MsgKey::FileStreamFailedToOpen, &[path, &e.to_string()]))?;

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
        .map_err(|e| fmt_msg(MsgKey::WriteStreamFailedToCreate, &[path, &e.to_string()]))?;

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
                    .map_err(|e| fmt_msg(MsgKey::WriteStreamFailedToWrite, &[path, &e.to_string()]))?;
                count += 1;
            }
            None => break,
        }
    }

    Ok(Value::Integer(count))
}

// ============================================
// ファイルシステム操作
// ============================================

/// list-dir - ディレクトリ内のファイル・ディレクトリ一覧を取得
/// 引数: (path) または (path :pattern "*.txt" :recursive true)
/// オプション:
///   :pattern - グロブパターン（例: "*.txt", "**/*.rs"）
///   :recursive - 再帰的に検索するか（デフォルト: false）
pub fn native_list_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/list-dir", "1"]));
    }

    let dir_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["io/list-dir", "a string"])),
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // パターンオプション
    let pattern = opts.get("pattern")
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None
        });

    // recursiveオプション
    let recursive = opts.get("recursive")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None
        })
        .unwrap_or(false);

    // グロブパターンの構築
    let glob_pattern = if let Some(pat) = pattern {
        if recursive {
            format!("{}/**/{}", dir_path, pat)
        } else {
            format!("{}/{}", dir_path, pat)
        }
    } else {
        if recursive {
            format!("{}/**/*", dir_path)
        } else {
            format!("{}/*", dir_path)
        }
    };

    // グロブでファイル一覧を取得
    let entries: Result<Vec<Value>, String> = glob::glob(&glob_pattern)
        .map_err(|e| fmt_msg(MsgKey::IoListDirInvalidPattern, &[&glob_pattern, &e.to_string()]))?
        .map(|entry| {
            entry
                .map(|path| Value::String(path.to_string_lossy().to_string()))
                .map_err(|e| fmt_msg(MsgKey::IoListDirFailedToRead, &[&e.to_string()]))
        })
        .collect();

    Ok(Value::List(entries?))
}

/// create-dir - ディレクトリを作成
/// 引数: (path) または (path :parents true)
/// オプション:
///   :parents - 親ディレクトリも作成するか（デフォルト: true）
pub fn native_create_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/create-dir", "1"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["io/create-dir", "a string"])),
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // parentsオプション（デフォルトtrue）
    let parents = opts.get("parents")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None
        })
        .unwrap_or(true);

    if parents {
        fs::create_dir_all(path)
            .map_err(|e| fmt_msg(MsgKey::IoCreateDirFailed, &[path, &e.to_string()]))?;
    } else {
        fs::create_dir(path)
            .map_err(|e| fmt_msg(MsgKey::IoCreateDirFailed, &[path, &e.to_string()]))?;
    }

    Ok(Value::Nil)
}

/// delete-file - ファイルを削除
/// 引数: (path)
pub fn native_delete_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/delete-file", "1"]));
    }

    match &args[0] {
        Value::String(path) => {
            fs::remove_file(path)
                .map_err(|e| fmt_msg(MsgKey::IoDeleteFileFailed, &[path, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/delete-file", "a string"])),
    }
}

/// delete-dir - ディレクトリを削除
/// 引数: (path) または (path :recursive true)
/// オプション:
///   :recursive - 中身ごと削除するか（デフォルト: false）
pub fn native_delete_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/delete-dir", "1"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["io/delete-dir", "a string"])),
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // recursiveオプション
    let recursive = opts.get("recursive")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None
        })
        .unwrap_or(false);

    if recursive {
        fs::remove_dir_all(path)
            .map_err(|e| fmt_msg(MsgKey::IoDeleteDirFailed, &[path, &e.to_string()]))?;
    } else {
        fs::remove_dir(path)
            .map_err(|e| fmt_msg(MsgKey::IoDeleteDirFailed, &[path, &e.to_string()]))?;
    }

    Ok(Value::Nil)
}

/// copy-file - ファイルをコピー
/// 引数: (src, dst)
pub fn native_copy_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/copy-file", "2"]));
    }

    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            fs::copy(src, dst)
                .map_err(|e| fmt_msg(MsgKey::IoCopyFileFailed, &[src, dst, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::BothArgsMustBeStrings, &["io/copy-file"])),
    }
}

/// move-file - ファイルを移動（名前変更）
/// 引数: (src, dst)
pub fn native_move_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/move-file", "2"]));
    }

    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            fs::rename(src, dst)
                .map_err(|e| fmt_msg(MsgKey::IoMoveFileFailed, &[src, dst, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::BothArgsMustBeStrings, &["io/move-file"])),
    }
}

/// file-info - ファイル/ディレクトリのメタデータを取得
/// 引数: (path)
/// 戻り値: {:size 1024 :modified "2024-01-01" :is-dir false :is-file true}
pub fn native_file_info(args: &[Value]) -> Result<Value, String> {
    use std::collections::HashMap;
    use std::time::UNIX_EPOCH;

    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/file-info", "1"]));
    }

    match &args[0] {
        Value::String(path) => {
            let metadata = fs::metadata(path)
                .map_err(|e| fmt_msg(MsgKey::IoGetMetadataFailed, &[path, &e.to_string()]))?;

            let mut info = HashMap::new();

            // サイズ
            info.insert("size".to_string(), Value::Integer(metadata.len() as i64));

            // ファイルタイプ
            info.insert("is-dir".to_string(), Value::Bool(metadata.is_dir()));
            info.insert("is-file".to_string(), Value::Bool(metadata.is_file()));

            // 更新日時
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    info.insert("modified".to_string(), Value::Integer(duration.as_secs() as i64));
                }
            }

            Ok(Value::Map(info))
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/file-info", "a string"])),
    }
}

/// is-file? - ファイルかどうか判定
/// 引数: (path)
pub fn native_is_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/is-file?", "1"]));
    }

    match &args[0] {
        Value::String(path) => {
            let path_obj = Path::new(path);
            Ok(Value::Bool(path_obj.is_file()))
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/is-file?", "a string"])),
    }
}

/// is-dir? - ディレクトリかどうか判定
/// 引数: (path)
pub fn native_is_dir(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["io/is-dir?", "1"]));
    }

    match &args[0] {
        Value::String(path) => {
            let path_obj = Path::new(path);
            Ok(Value::Bool(path_obj.is_dir()))
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/is-dir?", "a string"])),
    }
}
