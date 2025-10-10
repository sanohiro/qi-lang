//! 文字列操作関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use uuid::Uuid;

/// str - 値を文字列に変換して連結
pub fn native_str(args: &[Value]) -> Result<Value, String> {
    let s = args
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => format!("{}", v),
        })
        .collect::<String>();
    Ok(Value::String(s))
}

/// split - 文字列を分割
pub fn native_split(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["split"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            let parts: Vec<Value> = s
                .split(sep.as_str())
                .map(|p| Value::String(p.to_string()))
                .collect();
            Ok(Value::Vector(parts))
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// join - リストを文字列に結合
pub fn native_join(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["join"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(sep), Value::List(items)) | (Value::String(sep), Value::Vector(items)) => {
            let strings: Result<Vec<String>, String> = items
                .iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Ok(format!("{}", v)),
                })
                .collect();
            Ok(Value::String(strings?.join(sep)))
        }
        _ => Err(msg(MsgKey::JoinStringAndList).to_string()),
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
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// starts-with? - 接頭辞判定
pub fn native_starts_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["starts-with?"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// ends-with? - 接尾辞判定
pub fn native_ends_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["ends-with?"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// index-of - 部分文字列の最初の出現位置
pub fn native_index_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["index-of"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => {
            match s.find(sub.as_str()) {
                Some(idx) => Ok(Value::Integer(idx as i64)),
                None => Ok(Value::Integer(-1)),
            }
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// last-index-of - 部分文字列の最後の出現位置
pub fn native_last_index_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["last-index-of"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => {
            match s.rfind(sub.as_str()) {
                Some(idx) => Ok(Value::Integer(idx as i64)),
                None => Ok(Value::Integer(-1)),
            }
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// slice - 部分文字列を取得
pub fn native_slice(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(MsgKey::Need2Or3Args, &["slice"]));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::Integer(start), Value::Integer(end)) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let start_idx = (*start).max(0).min(len) as usize;
            let end_idx = (*end).max(0).min(len) as usize;
            if start_idx > end_idx {
                return Ok(Value::String(String::new()));
            }
            Ok(Value::String(chars[start_idx..end_idx].iter().collect()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["slice", "string and two integers"])),
    }
}

/// take-str - 先頭から指定数の文字を取得
pub fn native_take_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["take-str"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(n)) => {
            let chars: Vec<char> = s.chars().collect();
            let take_count = (*n).max(0) as usize;
            Ok(Value::String(chars.iter().take(take_count).collect()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["take-str", "string and integer"])),
    }
}

/// drop-str - 先頭から指定数の文字を削除
pub fn native_drop_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["drop-str"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Integer(n)) => {
            let chars: Vec<char> = s.chars().collect();
            let drop_count = (*n).max(0) as usize;
            Ok(Value::String(chars.iter().skip(drop_count).collect()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["drop-str", "string and integer"])),
    }
}

/// sub-before - 区切り文字より前の部分を取得
pub fn native_sub_before(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["sub-before"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(delim)) => {
            match s.find(delim.as_str()) {
                Some(idx) => Ok(Value::String(s[..idx].to_string())),
                None => Ok(Value::String(s.clone())),
            }
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
    }
}

/// sub-after - 区切り文字より後の部分を取得
pub fn native_sub_after(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["sub-after"]));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(delim)) => {
            match s.find(delim.as_str()) {
                Some(idx) => Ok(Value::String(s[idx + delim.len()..].to_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Err(msg(MsgKey::SplitTwoStrings).to_string()),
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
        (Value::String(s), Value::String(from), Value::String(to)) => {
            match s.find(from.as_str()) {
                Some(idx) => {
                    let mut result = String::new();
                    result.push_str(&s[..idx]);
                    result.push_str(to);
                    result.push_str(&s[idx + from.len()..]);
                    Ok(Value::String(result))
                }
                None => Ok(Value::String(s.clone())),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["replace-first", "three strings"])),
    }
}

/// lines - 改行で分割
pub fn native_lines(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["lines"]));
    }
    match &args[0] {
        Value::String(s) => {
            let lines: Vec<Value> = s
                .lines()
                .map(|line| Value::String(line.to_string()))
                .collect();
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
            let words: Vec<Value> = s
                .split_whitespace()
                .map(|word| Value::String(word.to_string()))
                .collect();
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
                return Err(fmt_msg(MsgKey::TypeOnly, &["repeat", "non-negative integer"]));
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
        Value::String(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["digit?", "strings"])),
    }
}

/// alpha? - 全てアルファベットか判定
pub fn native_alpha_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["alpha?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphabetic()))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["alpha?", "strings"])),
    }
}

/// alnum? - 全て英数字か判定
pub fn native_alnum_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["alnum?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphanumeric()))),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["alnum?", "strings"])),
    }
}

/// space? - 全て空白文字か判定
pub fn native_space_p(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["space?"]));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_whitespace()))),
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
            let all_lower = s.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase());
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
            let all_upper = s.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase());
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
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pad-left (3rd arg)", "single character"])),
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let chars: Vec<char> = s.chars().collect();
            if chars.len() >= width {
                return Ok(Value::String(s.clone()));
            }
            let pad_count = width - chars.len();
            let padded = pad_char.to_string().repeat(pad_count) + s;
            Ok(Value::String(padded))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["pad-left", "string and integer"])),
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
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pad-right (3rd arg)", "single character"])),
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let chars: Vec<char> = s.chars().collect();
            if chars.len() >= width {
                return Ok(Value::String(s.clone()));
            }
            let pad_count = width - chars.len();
            let mut padded = s.clone();
            padded.push_str(&pad_char.to_string().repeat(pad_count));
            Ok(Value::String(padded))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["pad-right", "string and integer"])),
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
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pad (3rd arg)", "single character"])),
                }
            } else {
                ' '
            };
            let width = (*width).max(0) as usize;
            let chars: Vec<char> = s.chars().collect();
            if chars.len() >= width {
                return Ok(Value::String(s.clone()));
            }
            let total_pad = width - chars.len();
            let left_pad = total_pad / 2;
            let right_pad = total_pad - left_pad;
            let padded = pad_char.to_string().repeat(left_pad) + s + &pad_char.to_string().repeat(right_pad);
            Ok(Value::String(padded))
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
            let squished = s
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            Ok(Value::String(squished))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["squish", "strings"])),
    }
}

/// expand-tabs - タブをスペースに変換
pub fn native_expand_tabs(args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["expand-tabs"]));
    }
    match &args[0] {
        Value::String(s) => {
            let tab_width = if args.len() == 2 {
                match &args[1] {
                    Value::Integer(n) if *n > 0 => *n as usize,
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["expand-tabs (2nd arg)", "positive integer"])),
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
            let chars: Vec<Value> = s
                .chars()
                .map(|c| Value::String(c.to_string()))
                .collect();
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
            let result = words.iter()
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
            let result = words.iter()
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
            let result: Vec<Value> = words.iter()
                .map(|w| Value::String(w.clone()))
                .collect();
            Ok(Value::Vector(result))
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
                let truncated: String = chars.iter().take(max_len.saturating_sub(suffix.len())).collect();
                Ok(Value::String(truncated + &suffix))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["truncate", "string and integer"])),
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
                    _ => return Err(fmt_msg(MsgKey::TypeOnly, &["trunc-words (3rd arg)", "string"])),
                }
            } else {
                "...".to_string()
            };

            let max_words = (*max_words).max(0) as usize;
            let words: Vec<&str> = s.split_whitespace().collect();

            if words.len() <= max_words {
                Ok(Value::String(s.clone()))
            } else {
                let truncated = words.iter().take(max_words).copied().collect::<Vec<_>>().join(" ");
                Ok(Value::String(truncated + &suffix))
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["trunc-words", "string and integer"])),
    }
}

/// splice - 位置ベースの置換
pub fn native_splice(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["splice", "4"]));
    }
    match (&args[0], &args[1], &args[2], &args[3]) {
        (Value::String(s), Value::Integer(start), Value::Integer(end), Value::String(replacement)) => {
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
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["splice", "string, two integers, and string"])),
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

            let result = s.lines()
                .map(|line| if line.is_empty() { line.to_string() } else { indent.clone() + line })
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
        Value::String(s) => {
            match s.trim().parse::<i64>() {
                Ok(n) => Ok(Value::Integer(n)),
                Err(_) => Err(fmt_msg(MsgKey::TypeOnly, &["parse-int", "valid integer string"])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["parse-int", "strings"])),
    }
}

/// parse-float - 文字列を浮動小数点数に変換
pub fn native_parse_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["parse-float"]));
    }
    match &args[0] {
        Value::String(s) => {
            match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Float(n)),
                Err(_) => Err(fmt_msg(MsgKey::TypeOnly, &["parse-float", "valid float string"])),
            }
        }
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
pub fn native_from_base64(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["from-base64"]));
    }
    match &args[0] {
        Value::String(s) => {
            match general_purpose::STANDARD.decode(s) {
                Ok(bytes) => {
                    match String::from_utf8(bytes) {
                        Ok(result) => Ok(Value::String(result)),
                        Err(_) => Err(fmt_msg(MsgKey::TypeOnly, &["from-base64", "valid UTF-8 bytes"])),
                    }
                }
                Err(_) => Err(fmt_msg(MsgKey::TypeOnly, &["from-base64", "valid base64 string"])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["from-base64", "strings"])),
    }
}

/// url-encode - URLエンコード
pub fn native_url_encode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["url-encode"]));
    }
    match &args[0] {
        Value::String(s) => {
            Ok(Value::String(urlencoding::encode(s).to_string()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["url-encode", "strings"])),
    }
}

/// url-decode - URLデコード
pub fn native_url_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["url-decode"]));
    }
    match &args[0] {
        Value::String(s) => {
            match urlencoding::decode(s) {
                Ok(decoded) => Ok(Value::String(decoded.to_string())),
                Err(_) => Err(fmt_msg(MsgKey::TypeOnly, &["url-decode", "valid URL-encoded string"])),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["url-decode", "strings"])),
    }
}

/// html-encode - HTMLエンコード
pub fn native_html_encode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["html-encode"]));
    }
    match &args[0] {
        Value::String(s) => {
            Ok(Value::String(html_escape::encode_text(s).to_string()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["html-encode", "strings"])),
    }
}

/// html-decode - HTMLデコード
pub fn native_html_decode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["html-decode"]));
    }
    match &args[0] {
        Value::String(s) => {
            Ok(Value::String(html_escape::decode_html_entities(s).to_string()))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["html-decode", "strings"])),
    }
}

/// hash - ハッシュ生成
pub fn native_hash(args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
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
