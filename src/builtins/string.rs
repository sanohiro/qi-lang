//! 文字列操作関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

#[cfg(feature = "string-encoding")]
use base64::{engine::general_purpose, Engine as _};

#[cfg(feature = "string-crypto")]
use sha2::{Digest, Sha256};

#[cfg(feature = "string-crypto")]
use uuid::Uuid;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use regex::Regex;

// ========================================
// Regexキャッシュ（グローバル）
// ========================================

/// 正規表現パターンのキャッシュ（5〜10倍の高速化）
static REGEX_CACHE: Lazy<DashMap<String, Regex>> = Lazy::new(DashMap::new);

/// Regexキャッシュから取得または新規コンパイル
fn get_or_compile_regex(pattern: &str) -> Result<Regex, regex::Error> {
    // キャッシュヒット時はcloneして返す
    if let Some(re) = REGEX_CACHE.get(pattern) {
        return Ok(re.clone());
    }

    // キャッシュミス時はコンパイルしてキャッシュに追加
    let re = Regex::new(pattern)?;
    REGEX_CACHE.insert(pattern.to_string(), re.clone());
    Ok(re)
}

// ========================================
// 文字列操作関数
// ========================================

/// split - 文字列を分割
pub fn native_split(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["split"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            // 中間Vecを排除してim::Vector::from_iterを直接使用
            let parts =
                im::Vector::from_iter(s.split(sep.as_str()).map(|p| Value::String(p.to_string())));
            Ok(Value::Vector(parts))
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// upper - 文字列を大文字に変換
pub fn native_upper(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["upper"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["upper", "strings"])),
    }
}

/// lower - 文字列を小文字に変換
pub fn native_lower(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["lower"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["lower", "strings"])),
    }
}

/// trim - 文字列の前後の空白を削除
pub fn native_trim(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["trim"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["trim", "strings"])),
    }
}

/// contains? - 部分文字列を含むか判定
pub fn native_contains(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["contains?"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub.as_str()))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["contains?", "two strings"])),
    }
}

/// starts-with? - 接頭辞判定
pub fn native_starts_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["starts-with?"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => {
            Ok(Value::Bool(s.starts_with(prefix.as_str())))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["starts-with?", "two strings"])),
    }
}

/// ends-with? - 接尾辞判定
pub fn native_ends_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["ends-with?"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["ends-with?", "two strings"])),
    }
}

/// index-of - 部分文字列の最初の出現位置
pub fn native_index_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["index-of"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => match s.find(sub.as_str()) {
            Some(idx) => Ok(Value::Integer(idx as i64)),
            None => Ok(Value::Nil),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["index-of", "two strings"])),
    }
}

/// last-index-of - 部分文字列の最後の出現位置
pub fn native_last_index_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["last-index-of"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => match s.rfind(sub.as_str()) {
            Some(idx) => Ok(Value::Integer(idx as i64)),
            None => Ok(Value::Nil),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["last-index-of", "two strings"])),
    }
}

/// slice - 部分文字列を取得
pub fn native_slice(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["slice"]));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::Integer(start), Value::Integer(end)) => {
            let char_count = s.chars().count() as i64;
            let start_idx = (*start).max(0).min(char_count) as usize;
            let end_idx = (*end).max(0).min(char_count) as usize;

            if start_idx >= end_idx {
                return Ok(Value::String(String::new()));
            }

            // char_indices()でバイトオフセットを取得
            let mut indices: Vec<(usize, char)> = s.char_indices().collect();
            indices.push((s.len(), '\0')); // 終端用

            let byte_start = indices[start_idx].0;
            let byte_end = indices[end_idx].0;

            Ok(Value::String(s[byte_start..byte_end].to_string()))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["slice", "string and two integers"],
        )),
    }
}

/// take-str - 先頭から指定数の文字を取得
pub fn native_take_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["take-str"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(n), Value::String(s)) => {
            let take_count = (*n).max(0) as usize;

            // char_indices()を使ってtake_count番目の文字のバイトオフセットを取得
            if let Some((byte_idx, _)) = s.char_indices().nth(take_count) {
                Ok(Value::String(s[..byte_idx].to_string()))
            } else {
                // take_countが文字列の長さを超える場合は全体を返す
                Ok(Value::String(s.clone()))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["take-str", "integer and string"],
        )),
    }
}

/// drop-str - 先頭から指定数の文字を削除
pub fn native_drop_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["drop-str"]));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(n), Value::String(s)) => {
            let drop_count = (*n).max(0) as usize;

            // char_indices()を使ってdrop_count番目の文字のバイトオフセットを取得
            if let Some((byte_idx, _)) = s.char_indices().nth(drop_count) {
                Ok(Value::String(s[byte_idx..].to_string()))
            } else {
                // drop_countが文字列の長さを超える場合は空文字列を返す
                Ok(Value::String(String::new()))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["drop-str", "integer and string"],
        )),
    }
}

/// sub-before - 区切り文字より前の部分を取得
pub fn native_sub_before(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["sub-before"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(delim)) => match s.find(delim.as_str()) {
            Some(idx) => Ok(Value::String(s[..idx].to_string())),
            None => Ok(Value::String(s.clone())),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sub-before", "two strings"])),
    }
}

/// sub-after - 区切り文字より後の部分を取得
pub fn native_sub_after(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["sub-after"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(delim)) => match s.find(delim.as_str()) {
            Some(idx) => Ok(Value::String(s[idx + delim.len()..].to_string())),
            None => Ok(Value::String(String::new())),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sub-after", "two strings"])),
    }
}

/// replace - 全ての一致部分を置換
pub fn native_replace(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["replace"]));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::String(from), Value::String(to)) => {
            Ok(Value::String(s.replace(from.as_str(), to.as_str())))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["replace", "three strings"])),
    }
}

/// replace-first - 最初の一致部分のみ置換
pub fn native_replace_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["replace-first"]));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::String(from), Value::String(to)) => match s.find(from.as_str()) {
            Some(idx) => {
                let mut result = String::new();
                result.push_str(&s[..idx]);
                result.push_str(to);
                result.push_str(&s[idx + from.len()..]);
                Ok(Value::String(result))
            }
            None => Ok(Value::String(s.clone())),
        },
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["replace-first", "three strings"],
        )),
    }
}

/// lines - 改行で分割
pub fn native_lines(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["lines"]));
    }
    match &args[0] {
        Value::String(s) => {
            // 中間Vecを排除してim::Vector::from_iterを直接使用
            let lines =
                im::Vector::from_iter(s.lines().map(|line| Value::String(line.to_string())));
            Ok(Value::Vector(lines))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["lines", "strings"])),
    }
}

/// words - 空白で分割
pub fn native_words(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["words"]));
    }
    match &args[0] {
        Value::String(s) => {
            // 中間Vecを排除してim::Vector::from_iterを直接使用
            let words = im::Vector::from_iter(
                s.split_whitespace()
                    .map(|word| Value::String(word.to_string())),
            );
            Ok(Value::Vector(words))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["words", "strings"])),
    }
}

/// capitalize - 先頭文字を大文字に
pub fn native_capitalize(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["capitalize"]));
    }
    match &args[0] {
        Value::String(s) => {
            let mut chars = s.chars();
            match chars.next() {
                None => Ok(Value::String(String::new())),
                Some(first) => {
                    let capitalized = first.to_uppercase().collect::<String>() + chars.as_str();
                    Ok(Value::String(capitalized))
                }
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["capitalize", "strings"])),
    }
}

/// trim-left - 左側の空白を削除
pub fn native_trim_left(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["trim-left"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim_start().to_string())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["trim-left", "strings"])),
    }
}

/// trim-right - 右側の空白を削除
pub fn native_trim_right(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["trim-right"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim_end().to_string())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["trim-right", "strings"])),
    }
}

/// repeat - 文字列を繰り返す
pub fn native_repeat(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["repeat"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(n)) => {
            if *n < 0 {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["repeat", "non-negative integer"],
                ));
            }
            Ok(Value::String(s.repeat(*n as usize)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["repeat", "string and integer"])),
    }
}

/// chars-count - Unicode文字数
pub fn native_chars_count(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["chars-count"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["chars-count", "strings"])),
    }
}

/// bytes-count - バイト数
pub fn native_bytes_count(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["bytes-count"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["bytes-count", "strings"])),
    }
}

/// digit? - 全て数字か判定
pub fn native_digit_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["digit?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()),
        )),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["digit?", "strings"])),
    }
}

/// alpha? - 全てアルファベットか判定
pub fn native_alpha_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["alpha?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_alphabetic()),
        )),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["alpha?", "strings"])),
    }
}

/// alnum? - 全て英数字か判定
pub fn native_alnum_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["alnum?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_alphanumeric()),
        )),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["alnum?", "strings"])),
    }
}

/// space? - 全て空白文字か判定
pub fn native_space_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["space?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_whitespace()),
        )),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["space?", "strings"])),
    }
}

/// lower? - 全て小文字か判定
pub fn native_lower_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["lower?"]));
    }
    match &args[0] {
        Value::String(s) => {
            let has_alpha = s.chars().any(|c| c.is_alphabetic());
            let all_lower = s
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_lowercase());
            Ok(Value::Bool(has_alpha && all_lower))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["lower?", "strings"])),
    }
}

/// upper? - 全て大文字か判定
pub fn native_upper_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["upper?"]));
    }
    match &args[0] {
        Value::String(s) => {
            let has_alpha = s.chars().any(|c| c.is_alphabetic());
            let all_upper = s
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase());
            Ok(Value::Bool(has_alpha && all_upper))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["upper?", "strings"])),
    }
}

/// pad-left - 左側に文字を詰める
pub fn native_pad_left(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["pad-left"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(width)) => {
            let pad_char = if args.len() == 3 {
                match &args[2] {
                    Value::String(ch) if ch.chars().count() == 1 => ch.chars().next().unwrap(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["pad-left (3rd arg)", "single character"],
                        ))
                    }
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let char_count = s.chars().count();

            if char_count >= width {
                return Ok(Value::String(s.clone()));
            }

            let pad_count = width - char_count;
            // String::with_capacity + push_strで効率的に構築
            let mut result = String::with_capacity(width * pad_char.len_utf8());
            for _ in 0..pad_count {
                result.push(pad_char);
            }
            result.push_str(s);
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["pad-left", "string and integer"],
        )),
    }
}

/// pad-right - 右側に文字を詰める
pub fn native_pad_right(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["pad-right"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(width)) => {
            let pad_char = if args.len() == 3 {
                match &args[2] {
                    Value::String(ch) if ch.chars().count() == 1 => ch.chars().next().unwrap(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["pad-right (3rd arg)", "single character"],
                        ))
                    }
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let char_count = s.chars().count();

            if char_count >= width {
                return Ok(Value::String(s.clone()));
            }

            let pad_count = width - char_count;
            // String::with_capacity + push_strで効率的に構築
            let mut result = String::with_capacity(width * pad_char.len_utf8());
            result.push_str(s);
            for _ in 0..pad_count {
                result.push(pad_char);
            }
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["pad-right", "string and integer"],
        )),
    }
}

/// pad - 中央揃え
pub fn native_pad(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["pad"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(width)) => {
            let pad_char = if args.len() == 3 {
                match &args[2] {
                    Value::String(ch) if ch.chars().count() == 1 => ch.chars().next().unwrap(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["pad (3rd arg)", "single character"],
                        ))
                    }
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let char_count = s.chars().count();

            if char_count >= width {
                return Ok(Value::String(s.clone()));
            }

            let total_pad = width - char_count;
            let left_pad = total_pad / 2;
            let right_pad = total_pad - left_pad;

            // String::with_capacity + push_strで効率的に構築
            let mut result = String::with_capacity(width * pad_char.len_utf8());
            for _ in 0..left_pad {
                result.push(pad_char);
            }
            result.push_str(s);
            for _ in 0..right_pad {
                result.push(pad_char);
            }
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["pad", "string and integer"])),
    }
}

/// squish - 連続空白を1つに、前後trim
pub fn native_squish(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["squish"]));
    }
    match &args[0] {
        Value::String(s) => {
            let squished = s.split_whitespace().collect::<Vec<&str>>().join(" ");
            Ok(Value::String(squished))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["squish", "strings"])),
    }
}

/// expand-tabs - タブをスペースに変換
pub fn native_expand_tabs(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["expand-tabs"]));
    }
    match &args[0] {
        Value::String(s) => {
            let tab_width = if args.len() == 2 {
                match &args[1] {
                    Value::Integer(n) if *n > 0 => *n as usize,
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["expand-tabs (2nd arg)", "positive integer"],
                        ))
                    }
                }
            } else {
                4
            };
            let expanded = s.replace('\t', &" ".repeat(tab_width));
            Ok(Value::String(expanded))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["expand-tabs", "strings"])),
    }
}

/// title - 各単語の先頭を大文字に
pub fn native_title(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["title"]));
    }
    match &args[0] {
        Value::String(s) => {
            let result = s
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["title", "strings"])),
    }
}

/// reverse - 文字列を反転
pub fn native_reverse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["reverse"]));
    }
    match &args[0] {
        Value::String(s) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::String(reversed))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["reverse", "strings"])),
    }
}

/// chars - 文字列を文字のリストに分割
pub fn native_chars(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["chars"]));
    }
    match &args[0] {
        Value::String(s) => {
            // 中間Vecを排除してim::Vector::from_iterを直接使用
            let chars = im::Vector::from_iter(s.chars().map(|c| Value::String(c.to_string())));
            Ok(Value::Vector(chars))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["chars", "strings"])),
    }
}

// ケース変換のヘルパー関数
fn split_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;

    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            prev_lower = false;
        } else if ch.is_uppercase() {
            if prev_lower && !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            current.push(ch);
            prev_lower = false;
        } else {
            current.push(ch);
            prev_lower = ch.is_lowercase();
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// snake - スネークケースに変換
pub fn native_snake(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["snake"]));
    }
    match &args[0] {
        Value::String(s) => {
            let words = split_words(s);
            let result = words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("_");
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["snake", "strings"])),
    }
}

/// camel - キャメルケースに変換
pub fn native_camel(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["camel"]));
    }
    match &args[0] {
        Value::String(s) => {
            let words = split_words(s);
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                if i == 0 {
                    result.push_str(&word.to_lowercase());
                } else {
                    let mut chars = word.chars();
                    if let Some(first) = chars.next() {
                        result.push_str(&first.to_uppercase().collect::<String>());
                        result.push_str(&chars.as_str().to_lowercase());
                    }
                }
            }
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["camel", "strings"])),
    }
}

/// kebab - ケバブケースに変換
pub fn native_kebab(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["kebab"]));
    }
    match &args[0] {
        Value::String(s) => {
            let words = split_words(s);
            let result = words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("-");
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["kebab", "strings"])),
    }
}

/// pascal - パスカルケースに変換
pub fn native_pascal(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["pascal"]));
    }
    match &args[0] {
        Value::String(s) => {
            let words = split_words(s);
            let mut result = String::new();
            for word in words {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    result.push_str(&first.to_uppercase().collect::<String>());
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["pascal", "strings"])),
    }
}

/// split-camel - キャメルケースを単語に分割
pub fn native_split_camel(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["split-camel"]));
    }
    match &args[0] {
        Value::String(s) => {
            let words = split_words(s);
            let result: Vec<Value> = words.iter().map(|w| Value::String(w.clone())).collect();
            Ok(Value::Vector(result.into()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["split-camel", "strings"])),
    }
}

/// truncate - 指定長で切り詰め
pub fn native_truncate(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["truncate"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(max_len)) => {
            let suffix = if args.len() == 3 {
                match &args[2] {
                    Value::String(suf) => suf.clone(),
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["truncate (3rd arg)", "string"])),
                }
            } else {
                "...".to_string()
            };

            let max_len = (*max_len).max(0) as usize;
            let chars: Vec<char> = s.chars().collect();

            if chars.len() <= max_len {
                Ok(Value::String(s.clone()))
            } else {
                let truncated: String = chars
                    .iter()
                    .take(max_len.saturating_sub(suffix.len()))
                    .collect();
                Ok(Value::String(truncated + &suffix))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["truncate", "string and integer"],
        )),
    }
}

/// trunc-words - 単語数で切り詰め
pub fn native_trunc_words(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["trunc-words"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(max_words)) => {
            let suffix = if args.len() == 3 {
                match &args[2] {
                    Value::String(suf) => suf.clone(),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["trunc-words (3rd arg)", "string"],
                        ))
                    }
                }
            } else {
                "...".to_string()
            };

            let max_words = (*max_words).max(0) as usize;
            let words: Vec<&str> = s.split_whitespace().collect();

            if words.len() <= max_words {
                Ok(Value::String(s.clone()))
            } else {
                let truncated = words
                    .iter()
                    .take(max_words)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(Value::String(truncated + &suffix))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["trunc-words", "string and integer"],
        )),
    }
}

/// splice - 位置ベースの置換
pub fn native_splice(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["splice", "4"]));
    }
    match (&args[0], &args[1], &args[2], &args[3]) {
        (
            Value::String(s),
            Value::Integer(start),
            Value::Integer(end),
            Value::String(replacement),
        ) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let start_idx = (*start).max(0).min(len) as usize;
            let end_idx = (*end).max(0).min(len) as usize;

            let mut result = String::new();
            result.extend(chars.iter().take(start_idx));
            result.push_str(replacement);
            result.extend(chars.iter().skip(end_idx));
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["splice", "string, two integers, and string"],
        )),
    }
}

/// numeric? - 数値として解釈可能か判定
pub fn native_numeric_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["numeric?"]));
    }
    match &args[0] {
        Value::String(s) => {
            let is_numeric = s.parse::<f64>().is_ok();
            Ok(Value::Bool(is_numeric))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["numeric?", "strings"])),
    }
}

/// integer? - 整数として解釈可能か判定
pub fn native_integer_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["integer?"]));
    }
    match &args[0] {
        Value::String(s) => {
            let is_integer = s.parse::<i64>().is_ok();
            Ok(Value::Bool(is_integer))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["integer?", "strings"])),
    }
}

/// blank? - 空または空白のみか判定
pub fn native_blank_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["blank?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(s.trim().is_empty())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["blank?", "strings"])),
    }
}

/// ascii? - ASCII文字のみか判定
pub fn native_ascii_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["ascii?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(s.is_ascii())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["ascii?", "strings"])),
    }
}

/// indent - 各行にインデントを追加
pub fn native_indent(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["indent"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(n)) => {
            let indent_str = if args.len() == 3 {
                match &args[2] {
                    Value::String(ind) => ind.clone(),
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["indent (3rd arg)", "string"])),
                }
            } else {
                " ".to_string()
            };

            let indent_count = (*n).max(0) as usize;
            let indent = indent_str.repeat(indent_count);

            let result = s
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        line.to_string()
                    } else {
                        indent.clone() + line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Value::String(result))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["indent", "string and integer"])),
    }
}

/// wrap - 指定幅で改行
pub fn native_wrap(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["wrap"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(width)) => {
            let width = (*width).max(1) as usize;
            let words: Vec<&str> = s.split_whitespace().collect();
            let mut lines = Vec::new();
            let mut current_line = String::new();

            for word in words {
                if current_line.is_empty() {
                    current_line.push_str(word);
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line.clone());
                    current_line = word.to_string();
                }
            }

            if !current_line.is_empty() {
                lines.push(current_line);
            }

            Ok(Value::String(lines.join("\n")))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["wrap", "string and integer"])),
    }
}

/// parse-int - 文字列を整数に変換
pub fn native_parse_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["parse-int"]));
    }
    match &args[0] {
        Value::String(s) => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Value::Integer(n)),
            Err(_) => Err(fmt_msg(
                MsgKey::TypeOnly,
                &["parse-int", "valid integer string"],
            )),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["parse-int", "strings"])),
    }
}

/// parse-float - 文字列を浮動小数点数に変換
pub fn native_parse_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["parse-float"]));
    }
    match &args[0] {
        Value::String(s) => match s.trim().parse::<f64>() {
            Ok(n) => Ok(Value::Float(n)),
            Err(_) => Err(fmt_msg(
                MsgKey::TypeOnly,
                &["parse-float", "valid float string"],
            )),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["parse-float", "strings"])),
    }
}

/// slugify - URL用のスラッグに変換
pub fn native_slugify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["slugify"]));
    }
    match &args[0] {
        Value::String(s) => {
            let slug = s
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() {
                        c
                    } else if c.is_whitespace() || c == '-' || c == '_' {
                        '-'
                    } else {
                        '\0'
                    }
                })
                .filter(|&c| c != '\0')
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            Ok(Value::String(slug))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["slugify", "strings"])),
    }
}

/// word-count - 単語数をカウント
pub fn native_word_count(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["word-count"]));
    }
    match &args[0] {
        Value::String(s) => {
            let count = s.split_whitespace().count() as i64;
            Ok(Value::Integer(count))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["word-count", "strings"])),
    }
}

/// to-base64 - Base64エンコード
#[cfg(feature = "string-encoding")]
pub fn native_to_base64(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["to-base64"]));
    }
    match &args[0] {
        Value::String(s) => {
            let encoded = general_purpose::STANDARD.encode(s);
            Ok(Value::String(encoded))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["to-base64", "strings"])),
    }
}

/// from-base64 - Base64デコード
#[cfg(feature = "string-encoding")]
pub fn native_from_base64(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["from-base64"]));
    }
    match &args[0] {
        Value::String(s) => match general_purpose::STANDARD.decode(s) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(result) => Ok(Value::String(result)),
                Err(_) => Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["from-base64", "valid UTF-8 bytes"],
                )),
            },
            Err(_) => Err(fmt_msg(
                MsgKey::TypeOnly,
                &["from-base64", "valid base64 string"],
            )),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["from-base64", "strings"])),
    }
}

/// url-encode - URLエンコード
#[cfg(feature = "string-encoding")]
pub fn native_url_encode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["url-encode"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(urlencoding::encode(s).to_string())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["url-encode", "strings"])),
    }
}

/// url-decode - URLデコード
#[cfg(feature = "string-encoding")]
pub fn native_url_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["url-decode"]));
    }
    match &args[0] {
        Value::String(s) => match urlencoding::decode(s) {
            Ok(decoded) => Ok(Value::String(decoded.to_string())),
            Err(_) => Err(fmt_msg(
                MsgKey::TypeOnly,
                &["url-decode", "valid URL-encoded string"],
            )),
        },
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["url-decode", "strings"])),
    }
}

/// html-encode - HTMLエンコード
#[cfg(feature = "string-encoding")]
pub fn native_html_encode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["html-encode"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(html_escape::encode_text(s).to_string())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["html-encode", "strings"])),
    }
}

/// html-decode - HTMLデコード
#[cfg(feature = "string-encoding")]
pub fn native_html_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["html-decode"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(
            html_escape::decode_html_entities(s).to_string(),
        )),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["html-decode", "strings"])),
    }
}

/// hash - ハッシュ生成
#[cfg(feature = "string-crypto")]
pub fn native_hash(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["hash"]));
    }
    match &args[0] {
        Value::String(s) => {
            let algo = if args.len() == 2 {
                match &args[1] {
                    Value::Keyword(k) => k.as_str(),
                    _ => "sha256",
                }
            } else {
                "sha256"
            };

            let hash = match algo {
                "sha256" => {
                    let mut hasher = Sha256::new();
                    hasher.update(s.as_bytes());
                    format!("{:x}", hasher.finalize())
                }
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["hash (algorithm)", "sha256"])),
            };
            Ok(Value::String(hash))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["hash", "strings"])),
    }
}

/// uuid - UUID生成
#[cfg(feature = "string-crypto")]
pub fn native_uuid(args: &[Value]) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["uuid"]));
    }
    let version = if args.len() == 1 {
        match &args[0] {
            Value::Keyword(k) => k.as_str(),
            _ => "v4",
        }
    } else {
        "v4"
    };

    let uuid_str = match version {
        "v4" => Uuid::new_v4().to_string(),
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["uuid (version)", "v4"])),
    };
    Ok(Value::String(uuid_str))
}

/// re-find - 正規表現で最初のマッチを検索
pub fn native_re_find(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["re-find", "2", "(pattern, text)"],
        ));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-find", "pattern"])),
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-find", "text"])),
    };

    let re = get_or_compile_regex(pattern)
        .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["re-find", &e.to_string()]))?;

    match re.find(text) {
        Some(m) => Ok(Value::String(m.as_str().to_string())),
        None => Ok(Value::Nil),
    }
}

/// re-matches - 正規表現で全てのマッチを検索
pub fn native_re_matches(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["re-matches", "2", "(pattern, text)"],
        ));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-matches", "pattern"])),
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-matches", "text"])),
    };

    let re = get_or_compile_regex(pattern)
        .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["re-matches", &e.to_string()]))?;

    let matches: Vec<Value> = re
        .find_iter(text)
        .map(|m| Value::String(m.as_str().to_string()))
        .collect();

    Ok(Value::List(matches.into()))
}

/// re-replace - 正規表現で置換
pub fn native_re_replace(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["re-replace", "3", "(pattern, replacement, text)"],
        ));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-replace", "pattern"])),
    };

    let replacement = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["re-replace", "replacement"],
            ))
        }
    };

    let text = match &args[2] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-replace", "text"])),
    };

    let re = get_or_compile_regex(pattern)
        .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["re-replace", &e.to_string()]))?;

    Ok(Value::String(re.replace_all(text, replacement).to_string()))
}

/// re-match-groups - 正規表現マッチとキャプチャグループを取得
pub fn native_re_match_groups(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["re-match-groups", "2", "(pattern, text)"],
        ));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["re-match-groups", "pattern"],
            ))
        }
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-match-groups", "text"])),
    };

    let re = get_or_compile_regex(pattern)
        .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["re-match-groups", &e.to_string()]))?;

    match re.captures(text) {
        Some(caps) => {
            // 全体のマッチ
            let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");

            // キャプチャグループ（インデックス1から）
            let groups: Vec<Value> = (1..caps.len())
                .map(|i| {
                    caps.get(i)
                        .map(|m| Value::String(m.as_str().to_string()))
                        .unwrap_or(Value::Nil)
                })
                .collect();

            let mut result = std::collections::HashMap::new();
            result.insert("match".to_string(), Value::String(full_match.to_string()));
            result.insert("groups".to_string(), Value::Vector(groups.into()));

            Ok(Value::Map(result.into()))
        }
        None => Ok(Value::Nil),
    }
}

/// re-split - 正規表現パターンで文字列を分割
pub fn native_re_split(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["re-split", "2-3", "(pattern, text [, limit])"],
        ));
    }

    let pattern = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-split", "pattern"])),
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["re-split", "text"])),
    };

    let limit = if args.len() == 3 {
        match &args[2] {
            Value::Integer(n) if *n > 0 => Some(*n as usize),
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["re-split (3rd arg - limit)", "positive integer"],
                ))
            }
        }
    } else {
        None
    };

    let re = get_or_compile_regex(pattern)
        .map_err(|e| fmt_msg(MsgKey::InvalidRegex, &["re-split", &e.to_string()]))?;

    let parts: Vec<Value> = if let Some(n) = limit {
        re.splitn(text, n)
            .map(|s| Value::String(s.to_string()))
            .collect()
    } else {
        re.split(text)
            .map(|s| Value::String(s.to_string()))
            .collect()
    };

    Ok(Value::Vector(parts.into()))
}

/// format - 文字列フォーマット（簡易実装）
pub fn native_format(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["format", "1+", "(format string)"],
        ));
    }

    let fmt_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["format", "a string"])),
    };

    let mut result = fmt_str.clone();
    for (i, arg) in args[1..].iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        let value_str = match arg {
            Value::String(s) => s.clone(),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            _ => format!("{:?}", arg),
        };
        result = result.replace(&placeholder, &value_str);
    }

    Ok(Value::String(result))
}

/// format-decimal - 小数点桁数を指定してフォーマット
/// 使い方: (str/format-decimal number decimals) または number |> (str/format-decimal _ decimals)
pub fn native_format_decimal(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["format-decimal"]));
    }

    let number = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["format-decimal (1st arg - number)", "number"],
            ))
        }
    };

    let decimals = match &args[1] {
        Value::Integer(n) if *n >= 0 => *n as usize,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &[
                    "format-decimal (2nd arg - decimals)",
                    "non-negative integer",
                ],
            ))
        }
    };

    Ok(Value::String(format!("{:.prec$}", number, prec = decimals)))
}

/// format-comma - 3桁区切りでフォーマット
pub fn native_format_comma(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["format-comma"]));
    }

    let number = match &args[0] {
        Value::Integer(n) => {
            let s = n.to_string();
            let negative = s.starts_with('-');
            let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

            let mut result = String::new();
            for (i, ch) in digits.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(',');
                }
                result.push(ch);
            }

            let formatted: String = result.chars().rev().collect();
            if negative {
                format!("-{}", formatted)
            } else {
                formatted
            }
        }
        Value::Float(f) => {
            let s = f.to_string();
            let parts: Vec<&str> = s.split('.').collect();
            let int_part = parts[0];
            let decimal_part = parts.get(1).copied();

            let negative = int_part.starts_with('-');
            let digits: String = int_part.chars().filter(|c| c.is_ascii_digit()).collect();

            let mut result = String::new();
            for (i, ch) in digits.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(',');
                }
                result.push(ch);
            }

            let formatted: String = result.chars().rev().collect();
            let mut final_result = if negative {
                format!("-{}", formatted)
            } else {
                formatted
            };

            if let Some(dec) = decimal_part {
                final_result.push('.');
                final_result.push_str(dec);
            }

            final_result
        }
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["format-comma", "number"])),
    };

    Ok(Value::String(number))
}

/// format-percent - パーセント表示でフォーマット
/// 使い方: (str/format-percent number [decimals]) または number |> (str/format-percent [decimals])
pub fn native_format_percent(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["format-percent"]));
    }

    let (number, decimals) = if args.len() == 2 {
        // 2引数: (format-percent number decimals)
        let number = match &args[0] {
            Value::Integer(n) => (*n as f64) * 100.0,
            Value::Float(f) => f * 100.0,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["format-percent (1st arg - number)", "number"],
                ))
            }
        };
        let decimals = match &args[1] {
            Value::Integer(n) if *n >= 0 => *n as usize,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &[
                        "format-percent (2nd arg - decimals)",
                        "non-negative integer",
                    ],
                ))
            }
        };
        (number, decimals)
    } else {
        // 1引数: (format-percent number) - デフォルトで0桁
        let number = match &args[0] {
            Value::Integer(n) => (*n as f64) * 100.0,
            Value::Float(f) => f * 100.0,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["format-percent (1st arg - number)", "number"],
                ))
            }
        };
        (number, 0)
    };

    Ok(Value::String(format!(
        "{:.prec$}%",
        number,
        prec = decimals
    )))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（feature-gatedでない関数のみ）
/// @qi-doc:category string
/// @qi-doc:note 60+ string manipulation functions
pub const FUNCTIONS: super::NativeFunctions = &[
    ("str/split", native_split),
    ("str/upper", native_upper),
    ("str/lower", native_lower),
    ("str/trim", native_trim),
    ("str/contains?", native_contains),
    ("str/starts-with?", native_starts_with),
    ("str/ends-with?", native_ends_with),
    ("str/index-of", native_index_of),
    ("str/last-index-of", native_last_index_of),
    ("str/slice", native_slice),
    ("str/take-str", native_take_str),
    ("str/drop-str", native_drop_str),
    ("str/sub-before", native_sub_before),
    ("str/sub-after", native_sub_after),
    ("str/replace", native_replace),
    ("str/replace-first", native_replace_first),
    ("str/lines", native_lines),
    ("str/words", native_words),
    ("str/capitalize", native_capitalize),
    ("str/trim-left", native_trim_left),
    ("str/trim-right", native_trim_right),
    ("str/repeat", native_repeat),
    ("str/chars-count", native_chars_count),
    ("str/bytes-count", native_bytes_count),
    ("str/digit?", native_digit_p),
    ("str/alpha?", native_alpha_p),
    ("str/alnum?", native_alnum_p),
    ("str/space?", native_space_p),
    ("str/lower?", native_lower_p),
    ("str/upper?", native_upper_p),
    ("str/pad-left", native_pad_left),
    ("str/pad-right", native_pad_right),
    ("str/pad", native_pad),
    ("str/squish", native_squish),
    ("str/expand-tabs", native_expand_tabs),
    ("str/title", native_title),
    ("str/reverse", native_reverse),
    ("str/chars", native_chars),
    ("str/snake", native_snake),
    ("str/camel", native_camel),
    ("str/kebab", native_kebab),
    ("str/pascal", native_pascal),
    ("str/split-camel", native_split_camel),
    ("str/truncate", native_truncate),
    ("str/trunc-words", native_trunc_words),
    ("str/splice", native_splice),
    ("str/numeric?", native_numeric_p),
    ("str/integer?", native_integer_p),
    ("str/blank?", native_blank_p),
    ("str/ascii?", native_ascii_p),
    ("str/indent", native_indent),
    ("str/wrap", native_wrap),
    ("str/parse-int", native_parse_int),
    ("str/parse-float", native_parse_float),
    ("str/slugify", native_slugify),
    ("str/word-count", native_word_count),
    ("str/re-find", native_re_find),
    ("str/re-matches", native_re_matches),
    ("str/re-replace", native_re_replace),
    ("str/re-match-groups", native_re_match_groups),
    ("str/re-split", native_re_split),
    ("str/format", native_format),
    ("str/format-decimal", native_format_decimal),
    ("str/format-comma", native_format_comma),
    ("str/format-percent", native_format_percent),
];

/// Feature-gated関数のリスト (string-encoding feature)
#[cfg(feature = "string-encoding")]
pub const FUNCTIONS_STRING_ENCODING: super::NativeFunctions = &[
    ("str/to-base64", native_to_base64),
    ("str/from-base64", native_from_base64),
    ("str/url-encode", native_url_encode),
    ("str/url-decode", native_url_decode),
    ("str/html-encode", native_html_encode),
    ("str/html-decode", native_html_decode),
];

/// Feature-gated関数のリスト (string-crypto feature)
#[cfg(feature = "string-crypto")]
pub const FUNCTIONS_STRING_CRYPTO: super::NativeFunctions =
    &[("str/hash", native_hash), ("str/uuid", native_uuid)];
