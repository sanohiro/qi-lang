//! ファイルI/O関数

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "encoding-extended")]
use encoding_rs::{
    Encoding,
    BIG5, // 中国語（簡体字・繁体字）
    EUC_JP,
    EUC_KR, // 韓国語
    GB18030,
    GBK,
    ISO_2022_JP, // 日本語
    SHIFT_JIS,
    UTF_16BE,
    UTF_16LE,
    UTF_8,
    WINDOWS_1251, // 欧州（西欧・ロシア）
    WINDOWS_1252,
};

// ============================================
// エンコーディング処理ユーティリティ
// ============================================

/// エンコーディングを解決
#[cfg(feature = "encoding-extended")]
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
#[cfg(feature = "encoding-extended")]
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
#[cfg(feature = "encoding-extended")]
fn decode_bytes(bytes: &[u8], encoding: &'static Encoding, path: &str) -> Result<String, String> {
    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        Err(fmt_msg(
            MsgKey::IoFailedToDecodeAs,
            &[path, encoding.name()],
        ))
    } else {
        Ok(decoded.into_owned())
    }
}

/// 自動エンコーディング検出（ベストエフォート）
#[cfg(feature = "encoding-extended")]
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
#[cfg(feature = "encoding-extended")]
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
fn parse_keyword_args(
    args: &[Value],
    start_idx: usize,
) -> Result<std::collections::HashMap<String, Value>, String> {
    let mut opts = std::collections::HashMap::new();
    let mut i = start_idx;

    while i < args.len() {
        match &args[i] {
            Value::Keyword(key) => {
                if i + 1 >= args.len() {
                    return Err(fmt_msg(MsgKey::KeywordRequiresValue, &[key]));
                }
                opts.insert(key.to_string(), args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::ExpectedKeywordArg,
                    &[&format!("{:?}", args[i])],
                ));
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
mod basic;
mod ops;
mod stdin;
mod stream;

pub use basic::*;
pub use ops::*;
pub use stdin::*;
pub use stream::*;
pub const FUNCTIONS: super::NativeFunctions = &[
    ("io/read-file", native_read_file),
    ("io/write-file", native_write_file),
    ("io/append-file", native_append_file),
    ("io/read-lines", native_read_lines),
    ("io/file-exists?", native_file_exists),
    ("io/file-stream", native_file_stream),
    ("io/write-stream", native_write_stream),
    ("io/list-dir", native_list_dir),
    ("io/create-dir", native_create_dir),
    ("io/delete-file", native_delete_file),
    ("io/delete-dir", native_delete_dir),
    ("io/copy-file", native_copy_file),
    ("io/move-file", native_move_file),
    ("io/file-info", native_file_info),
    ("io/is-file?", native_is_file),
    ("io/is-dir?", native_is_dir),
    // 標準入力
    ("io/stdin-line", native_stdin_read_line),
    ("io/stdin-lines", native_stdin_read_lines),
];
