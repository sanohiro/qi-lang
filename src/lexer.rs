use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::value::FStringPart;

/// ソースコード上の位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Span {
            line,
            column,
            offset,
        }
    }
}

/// トークンの種類
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // リテラル
    Integer(i64),
    Float(f64),
    String(String),
    FString(Vec<FStringPart>), // f"hello {name}"
    Symbol(String),
    Keyword(String),
    True,
    False,
    Nil,

    // 括弧
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    // その他
    Quote,         // '
    Backquote,     // `
    Unquote,       // ,
    UnquoteSplice, // ,@
    At,            // @
    Arrow,         // ->
    FatArrow,      // =>
    Bar,           // | (or pattern用)
    Pipe,          // |>
    PipeRailway,   // |>?
    ParallelPipe,  // ||>
    AsyncPipe,     // ~>
    Ellipsis,      // ...

    // ファイル終端
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    #[allow(dead_code)]
    fn current_span(&self) -> Span {
        Span::new(self.line, self.column, self.pos)
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current() == Some(';') {
            while let Some(ch) = self.current() {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        self.advance(); // 先頭の "
        let mut result = String::new();

        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(c) => result.push(c),
                    None => return Err(msg(MsgKey::UnclosedString).to_string()),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(msg(MsgKey::UnclosedString).to_string())
    }

    /// 複数行文字列を読み取る: """..."""
    fn read_multiline_string(&mut self) -> Result<String, String> {
        // """ をスキップ
        self.advance(); // "
        self.advance(); // "
        self.advance(); // "

        let mut result = String::new();

        while let Some(ch) = self.current() {
            // """ で終了チェック
            if ch == '"' && self.peek(1) == Some('"') && self.peek(2) == Some('"') {
                self.advance(); // "
                self.advance(); // "
                self.advance(); // "
                return Ok(result);
            } else if ch == '\\' {
                // エスケープシーケンス処理
                self.advance();
                match self.current() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(c) => result.push(c),
                    None => return Err(msg(MsgKey::UnclosedString).to_string()),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(msg(MsgKey::UnclosedString).to_string())
    }

    fn read_fstring(&mut self) -> Result<Vec<FStringPart>, String> {
        self.advance(); // f
        self.advance(); // "
        let mut parts = Vec::new();
        let mut current_text = String::new();

        while let Some(ch) = self.current() {
            if ch == '"' {
                // 残ったテキストを追加
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(current_text));
                }
                self.advance();
                return Ok(parts);
            } else if ch == '{' {
                // テキスト部分を確定
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(current_text.clone()));
                    current_text.clear();
                }
                // {}内のコードを読み取る
                self.advance(); // {
                let mut code = String::new();
                let mut depth = 1;
                while let Some(ch) = self.current() {
                    if ch == '{' {
                        depth += 1;
                        code.push(ch);
                        self.advance();
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            self.advance(); // }
                            break;
                        }
                        code.push(ch);
                        self.advance();
                    } else {
                        code.push(ch);
                        self.advance();
                    }
                }
                if depth != 0 {
                    return Err(msg(MsgKey::FStringUnclosedBrace).to_string());
                }
                parts.push(FStringPart::Code(code));
            } else if ch == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => current_text.push('\n'),
                    Some('t') => current_text.push('\t'),
                    Some('r') => current_text.push('\r'),
                    Some('\\') => current_text.push('\\'),
                    Some('"') => current_text.push('"'),
                    Some('{') => current_text.push('{'),
                    Some('}') => current_text.push('}'),
                    Some(c) => current_text.push(c),
                    None => return Err(msg(MsgKey::UnclosedString).to_string()),
                }
                self.advance();
            } else {
                current_text.push(ch);
                self.advance();
            }
        }

        Err(msg(MsgKey::FStringUnclosed).to_string())
    }

    /// 複数行f-stringを読み取る: f"""..."""
    fn read_multiline_fstring(&mut self) -> Result<Vec<FStringPart>, String> {
        self.advance(); // f
        self.advance(); // "
        self.advance(); // "
        self.advance(); // "

        let mut parts = Vec::new();
        let mut current_text = String::new();

        while let Some(ch) = self.current() {
            // """ で終了チェック
            if ch == '"' && self.peek(1) == Some('"') && self.peek(2) == Some('"') {
                // 残ったテキストを追加
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(current_text));
                }
                self.advance(); // "
                self.advance(); // "
                self.advance(); // "
                return Ok(parts);
            } else if ch == '{' {
                // テキスト部分を確定
                if !current_text.is_empty() {
                    parts.push(FStringPart::Text(current_text.clone()));
                    current_text.clear();
                }
                // {}内のコードを読み取る
                self.advance(); // {
                let mut code = String::new();
                let mut depth = 1;
                while let Some(ch) = self.current() {
                    if ch == '{' {
                        depth += 1;
                        code.push(ch);
                        self.advance();
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            self.advance(); // }
                            break;
                        }
                        code.push(ch);
                        self.advance();
                    } else {
                        code.push(ch);
                        self.advance();
                    }
                }
                if depth != 0 {
                    return Err(msg(MsgKey::FStringUnclosedBrace).to_string());
                }
                parts.push(FStringPart::Code(code));
            } else if ch == '\\' {
                // エスケープシーケンス処理
                self.advance();
                match self.current() {
                    Some('n') => current_text.push('\n'),
                    Some('t') => current_text.push('\t'),
                    Some('r') => current_text.push('\r'),
                    Some('\\') => current_text.push('\\'),
                    Some('"') => current_text.push('"'),
                    Some('{') => current_text.push('{'),
                    Some('}') => current_text.push('}'),
                    Some(c) => current_text.push(c),
                    None => return Err(msg(MsgKey::UnclosedString).to_string()),
                }
                self.advance();
            } else {
                current_text.push(ch);
                self.advance();
            }
        }

        Err(msg(MsgKey::FStringUnclosed).to_string())
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut is_float = false;

        // 負号の処理
        if self.current() == Some('-') {
            num_str.push('-');
            self.advance();
        }

        while let Some(ch) = self.current() {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            num_str
                .parse()
                .map(Token::Float)
                .map_err(|_| fmt_msg(MsgKey::NumberLiteralInvalid, &[&num_str]))
        } else {
            num_str
                .parse()
                .map(Token::Integer)
                .map_err(|_| fmt_msg(MsgKey::NumberLiteralInvalid, &[&num_str]))
        }
    }

    fn read_symbol_or_keyword(&mut self) -> Result<Token, String> {
        let mut result = String::new();
        let is_keyword = self.current() == Some(':');

        if is_keyword {
            self.advance(); // :をスキップ
        }

        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || "+-*/%<>=!?_-&".contains(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_keyword {
            // 空のキーワードをチェック
            if result.is_empty() {
                return Err(msg(MsgKey::EmptyKeyword).to_string());
            }
            return Ok(Token::Keyword(result));
        }

        // 特殊なシンボルのチェック
        let token = match result.as_str() {
            "nil" => Token::Nil,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Symbol(result),
        };
        Ok(token)
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        loop {
            self.skip_whitespace();

            // コメントのスキップ
            if self.current() == Some(';') {
                self.skip_comment();
                continue;
            }

            match self.current() {
                None => return Ok(Token::Eof),
                Some('(') => {
                    self.advance();
                    return Ok(Token::LParen);
                }
                Some(')') => {
                    self.advance();
                    return Ok(Token::RParen);
                }
                Some('[') => {
                    self.advance();
                    return Ok(Token::LBracket);
                }
                Some(']') => {
                    self.advance();
                    return Ok(Token::RBracket);
                }
                Some('{') => {
                    self.advance();
                    return Ok(Token::LBrace);
                }
                Some('}') => {
                    self.advance();
                    return Ok(Token::RBrace);
                }
                Some('\'') => {
                    self.advance();
                    return Ok(Token::Quote);
                }
                Some('`') => {
                    self.advance();
                    return Ok(Token::Backquote);
                }
                Some(',') if self.peek(1) == Some('@') => {
                    self.advance(); // ,
                    self.advance(); // @
                    return Ok(Token::UnquoteSplice);
                }
                Some(',') => {
                    self.advance();
                    return Ok(Token::Unquote);
                }
                Some('@') => {
                    self.advance();
                    return Ok(Token::At);
                }
                // 複数行文字列: """..."""
                Some('"') if self.peek(1) == Some('"') && self.peek(2) == Some('"') => {
                    let s = self.read_multiline_string()?;
                    return Ok(Token::String(s));
                }
                Some('"') => {
                    let s = self.read_string()?;
                    return Ok(Token::String(s));
                }
                Some('|') if self.peek(1) == Some('|') && self.peek(2) == Some('>') => {
                    self.advance(); // |
                    self.advance(); // |
                    self.advance(); // >
                    return Ok(Token::ParallelPipe);
                }
                Some('|') if self.peek(1) == Some('>') && self.peek(2) == Some('?') => {
                    self.advance(); // |
                    self.advance(); // >
                    self.advance(); // ?
                    return Ok(Token::PipeRailway);
                }
                Some('|') if self.peek(1) == Some('>') => {
                    self.advance(); // |
                    self.advance(); // >
                    return Ok(Token::Pipe);
                }
                Some('|') => {
                    self.advance(); // |
                    return Ok(Token::Bar);
                }
                Some(ch) if ch.is_numeric() => {
                    return self.read_number();
                }
                Some('=') if self.peek(1) == Some('>') => {
                    self.advance(); // =
                    self.advance(); // >
                    return Ok(Token::FatArrow);
                }
                Some('~') if self.peek(1) == Some('>') => {
                    self.advance(); // ~
                    self.advance(); // >
                    return Ok(Token::AsyncPipe);
                }
                Some('-') if self.peek(1) == Some('>') => {
                    self.advance(); // -
                    self.advance(); // >
                    return Ok(Token::Arrow);
                }
                Some('-') if self.peek(1).is_some_and(|c| c.is_numeric()) => {
                    return self.read_number();
                }
                Some('.') if self.peek(1) == Some('.') && self.peek(2) == Some('.') => {
                    self.advance(); // .
                    self.advance(); // .
                    self.advance(); // .
                    return Ok(Token::Ellipsis);
                }
                // 複数行f-string: f"""..."""
                Some('f')
                    if self.peek(1) == Some('"')
                        && self.peek(2) == Some('"')
                        && self.peek(3) == Some('"') =>
                {
                    let parts = self.read_multiline_fstring()?;
                    return Ok(Token::FString(parts));
                }
                Some('f') if self.peek(1) == Some('"') => {
                    let parts = self.read_fstring()?;
                    return Ok(Token::FString(parts));
                }
                Some(':') => {
                    return self.read_symbol_or_keyword();
                }
                Some(ch) if ch.is_alphabetic() || "+-*/%<>=!?_-&".contains(ch) => {
                    return self.read_symbol_or_keyword();
                }
                Some(ch) => {
                    return Err(fmt_msg(MsgKey::UnexpectedChar, &[&ch.to_string()]));
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integers() {
        let mut lexer = Lexer::new("123 -456");
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(-456));
    }

    #[test]
    fn test_floats() {
        let mut lexer = Lexer::new("3.14 -2.5");
        assert_eq!(lexer.next_token().unwrap(), Token::Float(3.14));
        assert_eq!(lexer.next_token().unwrap(), Token::Float(-2.5));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" "world\n""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("world\n".to_string())
        );
    }

    #[test]
    fn test_symbols() {
        let mut lexer = Lexer::new("foo bar+ baz?");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Symbol("foo".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Symbol("bar+".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Symbol("baz?".to_string())
        );
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new(":name :age");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Keyword("name".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Keyword("age".to_string())
        );
    }

    #[test]
    fn test_special_values() {
        let mut lexer = Lexer::new("nil true false");
        assert_eq!(lexer.next_token().unwrap(), Token::Nil);
        assert_eq!(lexer.next_token().unwrap(), Token::True);
        assert_eq!(lexer.next_token().unwrap(), Token::False);
    }

    #[test]
    fn test_parens() {
        let mut lexer = Lexer::new("(+ 1 2)");
        assert_eq!(lexer.next_token().unwrap(), Token::LParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Symbol("+".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(1));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(2));
        assert_eq!(lexer.next_token().unwrap(), Token::RParen);
    }

    #[test]
    fn test_multiline_string() {
        let mut lexer = Lexer::new(
            r#""""hello
world""""#,
        );
        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "hello\nworld"),
            _ => panic!("Expected multiline string"),
        }
    }

    #[test]
    fn test_multiline_string_with_escape() {
        let mut lexer = Lexer::new(r#""""line1\nline2\tindented""""#);
        match lexer.next_token().unwrap() {
            Token::String(s) => assert_eq!(s, "line1\nline2\tindented"),
            _ => panic!("Expected multiline string with escapes"),
        }
    }

    #[test]
    fn test_multiline_fstring() {
        crate::i18n::init();
        let input = r#"f"""Hello, {name}
Welcome!""""#;
        let mut lexer = Lexer::new(input);
        match lexer.next_token().unwrap() {
            Token::FString(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts[0], FStringPart::Text("Hello, ".to_string()));
                assert_eq!(parts[1], FStringPart::Code("name".to_string()));
                assert_eq!(parts[2], FStringPart::Text("\nWelcome!".to_string()));
            }
            _ => panic!("Expected multiline f-string"),
        }
    }
}
