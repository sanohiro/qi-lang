//! 文字列操作関数

use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::Value;

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
