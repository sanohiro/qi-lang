//! トリビアを保持するフォーマッタ専用トークナイザ。
//!
//! 既存の評価器が利用する `lexer` はコメントや空白を破棄するため、
//! フォーマッタでは独自にトークン列を構築する必要がある。
//!
//! Qi の構文を変更した場合は `src/parser.rs` と `formatter::doc` も忘れずに更新すること。

use crate::i18n::{msg, MsgKey};

/// コメントの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Inline,
}

/// フォーマッタ用トークンの種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmtTokenKind {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Quote,
    Backquote,
    Unquote,
    UnquoteSplice,
    At,
    Arrow,
    FatArrow,
    Pipe,
    PipeRailway,
    ParallelPipe,
    AsyncPipe,
    Ellipsis,
    Keyword,
    Number,
    Symbol,
    StringLiteral,
    FStringLiteral,
    Comment(CommentKind),
    Whitespace,
    Other,
}

/// フォーマッタ用トークン。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FmtToken {
    pub kind: FmtTokenKind,
    pub lexeme: String,
    pub start: usize,
    pub end: usize,
}

impl FmtToken {
    fn new(kind: FmtTokenKind, lexeme: String, start: usize, end: usize) -> Self {
        Self {
            kind,
            lexeme,
            start,
            end,
        }
    }
}

/// 入力文字列をフォーマッタ用のトークン列に変換する。
pub fn tokenize(src: &str) -> Result<Vec<FmtToken>, String> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let len = src.len();

    while pos < len {
        if let Some((_, next)) = try_whitespace(src, pos) {
            tokens.push(FmtToken::new(
                FmtTokenKind::Whitespace,
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        if src[pos..].starts_with(';') {
            let (kind, next) = read_comment(src, pos);
            tokens.push(FmtToken::new(
                FmtTokenKind::Comment(kind),
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        if src[pos..].starts_with("\"\"\"") {
            let next = read_multiline_string(src, pos)?;
            tokens.push(FmtToken::new(
                FmtTokenKind::StringLiteral,
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        if src[pos..].starts_with('\"') {
            let next = read_string(src, pos)?;
            tokens.push(FmtToken::new(
                FmtTokenKind::StringLiteral,
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        if src[pos..].starts_with("f\"\"\"") {
            let next = read_multiline_fstring(src, pos)?;
            tokens.push(FmtToken::new(
                FmtTokenKind::FStringLiteral,
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        if src[pos..].starts_with("f\"") {
            let next = read_fstring(src, pos)?;
            tokens.push(FmtToken::new(
                FmtTokenKind::FStringLiteral,
                src[pos..next].to_string(),
                pos,
                next,
            ));
            pos = next;
            continue;
        }

        macro_rules! punct {
            ($pattern:expr, $kind:expr) => {
                if src[pos..].starts_with($pattern) {
                    let end = pos + $pattern.len();
                    tokens.push(FmtToken::new($kind, src[pos..end].to_string(), pos, end));
                    pos = end;
                    continue;
                }
            };
        }

        punct!("||>", FmtTokenKind::ParallelPipe);
        punct!("|>?", FmtTokenKind::PipeRailway);
        punct!("|>", FmtTokenKind::Pipe);
        punct!("~>", FmtTokenKind::AsyncPipe);
        punct!("->", FmtTokenKind::Arrow);
        punct!("=>", FmtTokenKind::FatArrow);
        punct!(",@", FmtTokenKind::UnquoteSplice);
        punct!("...", FmtTokenKind::Ellipsis);

        if let Some(ch) = peek_char(src, pos) {
            let ch_len = ch.len_utf8();
            let single = match ch {
                '(' => Some(FmtTokenKind::LParen),
                ')' => Some(FmtTokenKind::RParen),
                '[' => Some(FmtTokenKind::LBracket),
                ']' => Some(FmtTokenKind::RBracket),
                '{' => Some(FmtTokenKind::LBrace),
                '}' => Some(FmtTokenKind::RBrace),
                '\'' => Some(FmtTokenKind::Quote),
                '`' => Some(FmtTokenKind::Backquote),
                ',' => Some(FmtTokenKind::Unquote),
                '@' => Some(FmtTokenKind::At),
                _ => None,
            };

            if let Some(kind) = single {
                tokens.push(FmtToken::new(kind, ch.to_string(), pos, pos + ch_len));
                pos += ch_len;
                continue;
            }
        }

        if src[pos..].starts_with(':') {
            let end = read_symbol(src, pos + 1);
            tokens.push(FmtToken::new(
                FmtTokenKind::Keyword,
                src[pos..end].to_string(),
                pos,
                end,
            ));
            pos = end;
            continue;
        }

        if is_number_start(src, pos) {
            let end = read_number(src, pos);
            tokens.push(FmtToken::new(
                FmtTokenKind::Number,
                src[pos..end].to_string(),
                pos,
                end,
            ));
            pos = end;
            continue;
        }

        if is_symbol_start(src, pos) {
            let end = read_symbol(src, pos);
            tokens.push(FmtToken::new(
                FmtTokenKind::Symbol,
                src[pos..end].to_string(),
                pos,
                end,
            ));
            pos = end;
            continue;
        }

        // どれにも該当しない1文字を Other として扱う。
        let (ch, next_pos) = advance_char(src, pos).expect("pos < len ensures char");
        tokens.push(FmtToken::new(
            FmtTokenKind::Other,
            ch.to_string(),
            pos,
            next_pos,
        ));
        pos = next_pos;
    }

    Ok(tokens)
}

fn try_whitespace(src: &str, pos: usize) -> Option<(FmtTokenKind, usize)> {
    let rest = &src[pos..];
    let mut end = pos;
    for (offset, ch) in rest.char_indices() {
        if !ch.is_whitespace() {
            break;
        }
        end = pos + offset + ch.len_utf8();
    }
    if end > pos {
        Some((FmtTokenKind::Whitespace, end))
    } else {
        None
    }
}

fn read_comment(src: &str, pos: usize) -> (CommentKind, usize) {
    let kind = detect_comment_kind(src, pos);
    let rest = &src[pos..];
    let end = rest
        .find('\n')
        .map(|i| pos + i + 1)
        .unwrap_or_else(|| src.len());
    (kind, end)
}

fn read_string(src: &str, start: usize) -> Result<usize, String> {
    let mut pos = start + 1;
    while pos < src.len() {
        let (ch, next) = advance_char(src, pos).unwrap();
        if ch == '\\' {
            pos = next;
            if pos >= src.len() {
                return Err(msg(MsgKey::UnclosedString).to_string());
            }
            pos = advance_char(src, pos)
                .map(|(_, n)| n)
                .ok_or_else(|| msg(MsgKey::UnclosedString).to_string())?;
            continue;
        }
        if ch == '"' {
            return Ok(next);
        }
        pos = next;
    }
    Err(msg(MsgKey::UnclosedString).to_string())
}

fn read_multiline_string(src: &str, start: usize) -> Result<usize, String> {
    let mut pos = start + 3;
    while pos < src.len() {
        if src[pos..].starts_with("\"\"\"") {
            return Ok(pos + 3);
        }
        if src[pos..].starts_with('\\') {
            pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
            if pos < src.len() {
                pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
            }
            continue;
        }
        pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
    }
    Err(msg(MsgKey::UnclosedString).to_string())
}

fn read_fstring(src: &str, start: usize) -> Result<usize, String> {
    let mut pos = start + 2; // f"
    let mut escaped = false;
    while pos < src.len() {
        let (ch, next) = advance_char(src, pos).unwrap();
        if escaped {
            escaped = false;
            pos = next;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
                pos = next;
            }
            '"' => return Ok(next),
            _ => {
                pos = next;
            }
        }
    }
    Err(msg(MsgKey::FStringUnclosed).to_string())
}

fn read_multiline_fstring(src: &str, start: usize) -> Result<usize, String> {
    let mut pos = start + 4; // f"""
    while pos < src.len() {
        if src[pos..].starts_with("\"\"\"") {
            return Ok(pos + 3);
        }
        if src[pos..].starts_with('\\') {
            pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
            if pos < src.len() {
                pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
            }
            continue;
        }
        pos = advance_char(src, pos).map(|(_, n)| n).unwrap_or(src.len());
    }
    Err(msg(MsgKey::FStringUnclosed).to_string())
}

fn read_number(src: &str, start: usize) -> usize {
    let mut pos = start;
    let mut seen_dot = false;
    let mut first = true;
    while pos < src.len() {
        let (ch, next) = match advance_char(src, pos) {
            Some(pair) => pair,
            None => break,
        };
        if first && (ch == '-' || ch == '+') {
            pos = next;
            first = false;
            continue;
        }
        first = false;
        if ch.is_ascii_digit() {
            pos = next;
            continue;
        }
        if ch == '.' && !seen_dot {
            seen_dot = true;
            pos = next;
            continue;
        }
        break;
    }
    pos
}

fn read_symbol(src: &str, start: usize) -> usize {
    let mut pos = start;
    while pos < src.len() {
        let (ch, next) = match advance_char(src, pos) {
            Some(pair) => pair,
            None => break,
        };
        if is_symbol_char(ch) {
            pos = next;
        } else {
            break;
        }
    }
    pos
}

fn detect_comment_kind(src: &str, start: usize) -> CommentKind {
    let before = &src[..start];
    let line_start = before.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    if before[line_start..].trim().is_empty() {
        CommentKind::Line
    } else {
        CommentKind::Inline
    }
}

fn is_number_start(src: &str, pos: usize) -> bool {
    if let Some(ch) = peek_char(src, pos) {
        if ch.is_ascii_digit() {
            return true;
        }
        if ch == '-' {
            if let Some(next) = peek_char(src, pos + ch.len_utf8()) {
                return next.is_ascii_digit();
            }
        }
    }
    false
}

fn is_symbol_start(src: &str, pos: usize) -> bool {
    if let Some(ch) = peek_char(src, pos) {
        if ch.is_whitespace() {
            return false;
        }
        match ch {
            '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | ',' | ';' | ':' => false,
            _ => true,
        }
    } else {
        false
    }
}

fn is_symbol_char(ch: char) -> bool {
    !ch.is_whitespace()
        && !matches!(
            ch,
            '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | ',' | ';'
        )
}

fn advance_char(src: &str, pos: usize) -> Option<(char, usize)> {
    let ch = src[pos..].chars().next()?;
    Some((ch, pos + ch.len_utf8()))
}

fn peek_char(src: &str, pos: usize) -> Option<char> {
    src[pos..].chars().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n;

    #[test]
    fn tokenize_whitespace_and_comments() {
        i18n::init();
        let src = "  (def x 10)  ; inline\n;; comment\n";
        let tokens = tokenize(src).unwrap();
        assert!(matches!(
            tokens.first().unwrap().kind,
            FmtTokenKind::Whitespace
        ));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, FmtTokenKind::Comment(CommentKind::Inline))));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, FmtTokenKind::Comment(CommentKind::Line))));
    }

    #[test]
    fn tokenize_strings() {
        i18n::init();
        let src = "(println \"hello\" \"\"\"multi\"\"\" f\"name: {name}\")";
        let tokens = tokenize(src).unwrap();
        assert!(tokens.iter().any(|t| t.lexeme == "\"hello\""));
        assert!(tokens.iter().any(|t| t.lexeme == "\"\"\"multi\"\"\""));
        assert!(tokens.iter().any(|t| t.lexeme == "f\"name: {name}\""));
        assert!(
            tokens
                .iter()
                .filter(|t| matches!(t.kind, FmtTokenKind::StringLiteral))
                .count()
                >= 2
        );
    }

    #[test]
    fn tokenize_numbers_and_symbols() {
        i18n::init();
        let src = "(+ -1.25 value)";
        let tokens = tokenize(src).unwrap();
        assert!(tokens
            .iter()
            .any(|t| t.lexeme == "-1.25" && t.kind == FmtTokenKind::Number));
        assert!(tokens
            .iter()
            .any(|t| t.lexeme == "value" && t.kind == FmtTokenKind::Symbol));
        assert!(tokens.iter().any(|t| t.kind == FmtTokenKind::LParen));
    }
}
