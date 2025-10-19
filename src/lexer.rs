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

/// 位置情報付きトークン
#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub span: Span,
}

impl LocatedToken {
    pub fn new(token: Token, span: Span) -> Self {
        LocatedToken { token, span }
    }
}

/// トークンの種類
#[derive(Debug, Clone, PartialEq)]
/// トークン定義
/// @qi-doc:tokens
/// @qi-doc:pipe-operators |>, |>?, ||>, ~>
/// @qi-doc:arrow-operators ->, =>
/// @qi-doc:pattern-operators |
/// @qi-doc:quote-operators ', `, ,, ,@
/// @qi-doc:special-operators @, ...
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

impl Token {
    /// トークンをユーザーフレンドリーな文字列で表示
    pub fn display_name(&self) -> String {
        match self {
            Token::Integer(n) => n.to_string(),
            Token::Float(f) => f.to_string(),
            Token::String(s) => format!("\"{}\"", s),
            Token::FString(_) => "f-string".to_string(),
            Token::Symbol(s) => s.clone(),
            Token::Keyword(k) => format!(":{}", k),
            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::Nil => "nil".to_string(),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::LBracket => "[".to_string(),
            Token::RBracket => "]".to_string(),
            Token::LBrace => "{".to_string(),
            Token::RBrace => "}".to_string(),
            Token::Quote => "'".to_string(),
            Token::Backquote => "`".to_string(),
            Token::Unquote => ",".to_string(),
            Token::UnquoteSplice => ",@".to_string(),
            Token::At => "@".to_string(),
            Token::Arrow => "->".to_string(),
            Token::FatArrow => "=>".to_string(),
            Token::Bar => "|".to_string(),
            Token::Pipe => "|>".to_string(),
            Token::PipeRailway => "|>?".to_string(),
            Token::ParallelPipe => "||>".to_string(),
            Token::AsyncPipe => "~>".to_string(),
            Token::Ellipsis => "...".to_string(),
            Token::Eof => "EOF".to_string(),
        }
    }
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

    /// エスケープシーケンスを処理（共通ヘルパー）
    fn process_escape_sequence(&mut self) -> Result<char, String> {
        self.advance(); // \ をスキップ
        match self.current() {
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some(c) => {
                let ch = c;
                self.advance();
                Ok(ch)
            }
            None => Err(msg(MsgKey::UnclosedString).to_string()),
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        self.advance(); // 先頭の "
                        // 32: 一般的な文字列リテラル（変数名、短いメッセージ等）の平均長
                        // 例: "hello", "error message", "path/to/file" など
        let mut result = String::with_capacity(32);

        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                result.push(self.process_escape_sequence()?);
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

        // 128: 複数行文字列は通常ドキュメントやテンプレートなので長め
        // 例: docstring, SQLクエリ, HTMLテンプレート等
        let mut result = String::with_capacity(128);

        while let Some(ch) = self.current() {
            // """ で終了チェック
            if ch == '"' && self.peek(1) == Some('"') && self.peek(2) == Some('"') {
                self.advance(); // "
                self.advance(); // "
                self.advance(); // "
                return Ok(result);
            } else if ch == '\\' {
                result.push(self.process_escape_sequence()?);
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
                        // 4: f-stringは通常 [Text, Code, Text, ...] のパターンで2-4個
                        // 例: f"Hello {name}, you are {age} years old" → [Text, Code, Text, Code, Text]
        let mut parts = Vec::with_capacity(4);
        // 32: f-string内のテキスト部分は短いことが多い
        let mut current_text = String::with_capacity(32);

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
                                // 16: f-string内の式は通常短い（変数名や簡単な演算）
                                // 例: {name}, {age + 1}, {user.name} など
                let mut code = String::with_capacity(16);
                let mut depth = 1;
                let mut in_string = false;
                while let Some(ch) = self.current() {
                    if in_string {
                        // 文字列リテラル内
                        if ch == '\\' && self.peek(1) == Some('"') {
                            // \" は f-string のエスケープシーケンス（文字列リテラルの終了）
                            in_string = false;
                            self.advance(); // \ をスキップ
                            code.push('"'); // " を追加
                            self.advance();
                        } else if ch == '\\' {
                            // その他のエスケープシーケンス処理
                            code.push(ch);
                            self.advance();
                            if let Some(next_ch) = self.current() {
                                code.push(next_ch);
                                self.advance();
                            }
                        } else if ch == '"' {
                            // 文字列リテラルの終了（エスケープされていない "）
                            in_string = false;
                            code.push(ch);
                            self.advance();
                        } else {
                            code.push(ch);
                            self.advance();
                        }
                    } else {
                        // 文字列リテラル外
                        if ch == '\\' && self.peek(1) == Some('"') {
                            // \" は Qi ソースコードとしての " （文字列リテラルの開始）
                            in_string = true;
                            self.advance(); // \ をスキップ
                            code.push('"'); // " を追加
                            self.advance();
                        } else if ch == '"' {
                            // 文字列リテラルの開始
                            in_string = true;
                            code.push(ch);
                            self.advance();
                        } else if ch == '{' {
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
                }
                if depth != 0 {
                    return Err(msg(MsgKey::FStringUnclosedBrace).to_string());
                }
                parts.push(FStringPart::Code(code));
            } else if ch == '\\' {
                // { や } のエスケープは特殊処理
                if self.peek(1) == Some('{') || self.peek(1) == Some('}') {
                    self.advance(); // \ をスキップ
                    let special_ch = self.current().unwrap();
                    current_text.push(special_ch);
                    self.advance();
                } else {
                    current_text.push(self.process_escape_sequence()?);
                }
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

        // 4: f-stringは通常 [Text, Code, Text, ...] のパターンで2-4個
        let mut parts = Vec::with_capacity(4);
        // 128: 複数行f-stringなので長め（テンプレート等）
        let mut current_text = String::with_capacity(128);

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
                                // 16: f-string内の式は通常短い（変数名や簡単な演算）
                                // 例: {name}, {age + 1}, {user.name} など
                let mut code = String::with_capacity(16);
                let mut depth = 1;
                let mut in_string = false;
                while let Some(ch) = self.current() {
                    if in_string {
                        // 文字列リテラル内
                        if ch == '\\' && self.peek(1) == Some('"') {
                            // \" は f-string のエスケープシーケンス（文字列リテラルの終了）
                            in_string = false;
                            self.advance(); // \ をスキップ
                            code.push('"'); // " を追加
                            self.advance();
                        } else if ch == '\\' {
                            // その他のエスケープシーケンス処理
                            code.push(ch);
                            self.advance();
                            if let Some(next_ch) = self.current() {
                                code.push(next_ch);
                                self.advance();
                            }
                        } else if ch == '"' {
                            // 文字列リテラルの終了（エスケープされていない "）
                            in_string = false;
                            code.push(ch);
                            self.advance();
                        } else {
                            code.push(ch);
                            self.advance();
                        }
                    } else {
                        // 文字列リテラル外
                        if ch == '\\' && self.peek(1) == Some('"') {
                            // \" は Qi ソースコードとしての " （文字列リテラルの開始）
                            in_string = true;
                            self.advance(); // \ をスキップ
                            code.push('"'); // " を追加
                            self.advance();
                        } else if ch == '"' {
                            // 文字列リテラルの開始
                            in_string = true;
                            code.push(ch);
                            self.advance();
                        } else if ch == '{' {
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
                }
                if depth != 0 {
                    return Err(msg(MsgKey::FStringUnclosedBrace).to_string());
                }
                parts.push(FStringPart::Code(code));
            } else if ch == '\\' {
                // { や } のエスケープは特殊処理
                if self.peek(1) == Some('{') || self.peek(1) == Some('}') {
                    self.advance(); // \ をスキップ
                    let special_ch = self.current().unwrap();
                    current_text.push(special_ch);
                    self.advance();
                } else {
                    current_text.push(self.process_escape_sequence()?);
                }
            } else {
                current_text.push(ch);
                self.advance();
            }
        }

        Err(msg(MsgKey::FStringUnclosed).to_string())
    }

    fn read_number(&mut self, start_span: Span) -> Result<LocatedToken, String> {
        // 16: 64bit整数の最大桁数は約20桁、浮動小数点数も同程度
        // 例: 123, -456, 3.14159265358979, 9223372036854775807 (i64::MAX)
        let mut num_str = String::with_capacity(16);
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

        let token = if is_float {
            num_str
                .parse()
                .map(Token::Float)
                .map_err(|_| fmt_msg(MsgKey::NumberLiteralInvalid, &[&num_str]))?
        } else {
            num_str
                .parse()
                .map(Token::Integer)
                .map_err(|_| fmt_msg(MsgKey::NumberLiteralInvalid, &[&num_str]))?
        };
        Ok(LocatedToken::new(token, start_span))
    }

    fn read_symbol_or_keyword(&mut self, start_span: Span) -> Result<LocatedToken, String> {
        // 16: 関数名・変数名は通常10-15文字程度
        // 例: map, filter, reduce, define-function, http-get など
        let mut result = String::with_capacity(16);
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
            return Ok(LocatedToken::new(Token::Keyword(result), start_span));
        }

        // 特殊なシンボルのチェック
        let token = match result.as_str() {
            "nil" => Token::Nil,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Symbol(result),
        };
        Ok(LocatedToken::new(token, start_span))
    }

    pub fn next_token(&mut self) -> Result<LocatedToken, String> {
        loop {
            self.skip_whitespace();

            // コメントのスキップ
            if self.current() == Some(';') {
                self.skip_comment();
                continue;
            }

            let start_span = self.current_span();

            match self.current() {
                None => return Ok(LocatedToken::new(Token::Eof, start_span)),
                Some('(') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::LParen, start_span));
                }
                Some(')') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::RParen, start_span));
                }
                Some('[') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::LBracket, start_span));
                }
                Some(']') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::RBracket, start_span));
                }
                Some('{') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::LBrace, start_span));
                }
                Some('}') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::RBrace, start_span));
                }
                Some('\'') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::Quote, start_span));
                }
                Some('`') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::Backquote, start_span));
                }
                Some(',') if self.peek(1) == Some('@') => {
                    self.advance(); // ,
                    self.advance(); // @
                    return Ok(LocatedToken::new(Token::UnquoteSplice, start_span));
                }
                Some(',') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::Unquote, start_span));
                }
                Some('@') => {
                    self.advance();
                    return Ok(LocatedToken::new(Token::At, start_span));
                }
                // 複数行文字列: """..."""
                Some('"') if self.peek(1) == Some('"') && self.peek(2) == Some('"') => {
                    let s = self.read_multiline_string()?;
                    return Ok(LocatedToken::new(Token::String(s), start_span));
                }
                Some('"') => {
                    let s = self.read_string()?;
                    return Ok(LocatedToken::new(Token::String(s), start_span));
                }
                Some('|') if self.peek(1) == Some('|') && self.peek(2) == Some('>') => {
                    self.advance(); // |
                    self.advance(); // |
                    self.advance(); // >
                    return Ok(LocatedToken::new(Token::ParallelPipe, start_span));
                }
                Some('|') if self.peek(1) == Some('>') && self.peek(2) == Some('?') => {
                    self.advance(); // |
                    self.advance(); // >
                    self.advance(); // ?
                    return Ok(LocatedToken::new(Token::PipeRailway, start_span));
                }
                Some('|') if self.peek(1) == Some('>') => {
                    self.advance(); // |
                    self.advance(); // >
                    return Ok(LocatedToken::new(Token::Pipe, start_span));
                }
                Some('|') => {
                    self.advance(); // |
                    return Ok(LocatedToken::new(Token::Bar, start_span));
                }
                Some(ch) if ch.is_numeric() => {
                    return self.read_number(start_span);
                }
                Some('=') if self.peek(1) == Some('>') => {
                    self.advance(); // =
                    self.advance(); // >
                    return Ok(LocatedToken::new(Token::FatArrow, start_span));
                }
                Some('~') if self.peek(1) == Some('>') => {
                    self.advance(); // ~
                    self.advance(); // >
                    return Ok(LocatedToken::new(Token::AsyncPipe, start_span));
                }
                Some('-') if self.peek(1) == Some('>') => {
                    self.advance(); // -
                    self.advance(); // >
                    return Ok(LocatedToken::new(Token::Arrow, start_span));
                }
                Some('-') if self.peek(1).is_some_and(|c| c.is_numeric()) => {
                    return self.read_number(start_span);
                }
                Some('.') if self.peek(1) == Some('.') && self.peek(2) == Some('.') => {
                    self.advance(); // .
                    self.advance(); // .
                    self.advance(); // .
                    return Ok(LocatedToken::new(Token::Ellipsis, start_span));
                }
                // 複数行f-string: f"""..."""
                Some('f')
                    if self.peek(1) == Some('"')
                        && self.peek(2) == Some('"')
                        && self.peek(3) == Some('"') =>
                {
                    let parts = self.read_multiline_fstring()?;
                    return Ok(LocatedToken::new(Token::FString(parts), start_span));
                }
                Some('f') if self.peek(1) == Some('"') => {
                    let parts = self.read_fstring()?;
                    return Ok(LocatedToken::new(Token::FString(parts), start_span));
                }
                Some(':') => {
                    return self.read_symbol_or_keyword(start_span);
                }
                Some(ch) if ch.is_alphabetic() || "+-*/%<>=!?_-&".contains(ch) => {
                    return self.read_symbol_or_keyword(start_span);
                }
                Some(ch) => {
                    return Err(fmt_msg(MsgKey::UnexpectedChar, &[&ch.to_string()]));
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<LocatedToken>, String> {
        // トークン数推定: 平均的なLispコードでは5文字に1トークン程度
        // 例: "(defn add [a b] (+ a b))" → 20文字、7トークン（約1/3）
        // 空白・改行を含めると約1/5になる。最小16を保証。
        let estimated_tokens = (self.input.len() / 5).max(16);
        let mut tokens = Vec::with_capacity(estimated_tokens);
        loop {
            let loc_token = self.next_token()?;
            if loc_token.token == Token::Eof {
                break;
            }
            tokens.push(loc_token);
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
