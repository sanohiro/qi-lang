use crate::i18n::{fmt_msg, msg, MsgKey};
use crate::lexer::{Lexer, Token};
use crate::value::{Expr, MatchArm, Pattern, UseMode};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Parser { tokens, pos: 0 })
    }

    fn current(&self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.current() {
            Some(token) if token == &expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(fmt_msg(MsgKey::ExpectedToken, &[&format!("{:?}", expected), &format!("{:?}", token)])),
            None => Err(fmt_msg(MsgKey::ExpectedToken, &[&format!("{:?}", expected), "EOF"])),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expr()
    }

    pub fn parse_all(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs = Vec::new();
        while self.current().is_some() {
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        // パイプライン演算子を処理
        while self.current() == Some(&Token::Pipe) {
            self.advance();
            let right = self.parse_primary()?;

            // x |> f を (f x) に変換
            // x |> (f a b) を (f a b x) に変換
            expr = match right {
                // 右辺が関数呼び出しの場合、最後の引数に追加
                Expr::Call { func, mut args } => {
                    args.push(expr);
                    Expr::Call { func, args }
                }
                // それ以外は通常の呼び出し
                _ => Expr::Call {
                    func: Box::new(right),
                    args: vec![expr],
                },
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Nil) => {
                self.advance();
                Ok(Expr::Nil)
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Some(Token::Integer(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::Integer(n))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Ok(Expr::Float(f))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            Some(Token::FString(parts)) => {
                let parts = parts.clone();
                self.advance();
                Ok(Expr::FString(parts))
            }
            Some(Token::Symbol(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Symbol(s))
            }
            Some(Token::Keyword(k)) => {
                let k = k.clone();
                self.advance();
                Ok(Expr::Keyword(k))
            }
            Some(Token::LParen) => self.parse_list(),
            Some(Token::LBracket) => self.parse_vector(),
            Some(Token::LBrace) => self.parse_map(),
            Some(Token::Quote) => self.parse_quote(),
            Some(token) => Err(fmt_msg(MsgKey::UnexpectedToken, &[&format!("{:?}", token)])),
            None => Err(msg(MsgKey::UnexpectedEof).to_string()),
        }
    }

    fn parse_list(&mut self) -> Result<Expr, String> {
        self.expect(Token::LParen)?;

        // 空リスト
        if self.current() == Some(&Token::RParen) {
            self.advance();
            return Ok(Expr::List(vec![]));
        }

        // 特殊形式のチェック
        if let Some(Token::Symbol(name)) = self.current() {
            let name = name.clone();
            match name.as_str() {
                "def" => return self.parse_def(),
                "fn" => return self.parse_fn(),
                "let" => return self.parse_let(),
                "if" => return self.parse_if(),
                "do" => return self.parse_do(),
                "match" => return self.parse_match(),
                "try" => return self.parse_try(),
                "defer" => return self.parse_defer(),
                "module" => return self.parse_module(),
                "export" => return self.parse_export(),
                "use" => return self.parse_use(),
                _ => {}
            }
        }

        // 通常のリスト（関数呼び出し）
        let first_expr = self.parse_primary()?;  // パイプラインを含まない

        // パイプラインのチェック
        if self.current() == Some(&Token::Pipe) {
            let mut expr = first_expr;
            while self.current() == Some(&Token::Pipe) {
                self.advance();
                let right = self.parse_primary()?;

                expr = match right {
                    Expr::Call { func, mut args } => {
                        args.push(expr);
                        Expr::Call { func, args }
                    }
                    _ => Expr::Call {
                        func: Box::new(right),
                        args: vec![expr],
                    },
                };
            }
            self.expect(Token::RParen)?;
            return Ok(expr);
        }

        // 通常の関数呼び出し
        let func = Box::new(first_expr);
        let mut args = Vec::new();

        while self.current() != Some(&Token::RParen) {
            args.push(self.parse_expr()?);
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Call { func, args })
    }

    fn parse_def(&mut self) -> Result<Expr, String> {
        self.advance(); // 'def'をスキップ

        let name = match self.current() {
            Some(Token::Symbol(s)) => s.clone(),
            _ => return Err(fmt_msg(MsgKey::NeedsSymbol, &["def"]).to_string()),
        };
        self.advance();

        let value = Box::new(self.parse_expr()?);
        self.expect(Token::RParen)?;

        Ok(Expr::Def(name, value))
    }

    fn parse_fn(&mut self) -> Result<Expr, String> {
        self.advance(); // 'fn'をスキップ

        // パラメータリストのパース
        self.expect(Token::LBracket)?;
        let mut params = Vec::new();
        let mut is_variadic = false;

        while self.current() != Some(&Token::RBracket) {
            if let Some(Token::Symbol(s)) = self.current() {
                if s == "&" {
                    self.advance();
                    is_variadic = true;
                    if let Some(Token::Symbol(vararg)) = self.current() {
                        params.push(vararg.clone());
                        self.advance();
                    } else {
                        return Err(msg(MsgKey::VarargNeedsName).to_string());
                    }
                    break;
                } else {
                    params.push(s.clone());
                    self.advance();
                }
            } else {
                return Err(fmt_msg(MsgKey::NeedsSymbol, &["fn"]).to_string());
            }
        }

        self.expect(Token::RBracket)?;

        // 本体のパース
        let body = Box::new(self.parse_expr()?);
        self.expect(Token::RParen)?;

        Ok(Expr::Fn {
            params,
            body,
            is_variadic,
        })
    }

    fn parse_let(&mut self) -> Result<Expr, String> {
        self.advance(); // 'let'をスキップ

        // 束縛のパース
        self.expect(Token::LBracket)?;
        let mut bindings = Vec::new();

        while self.current() != Some(&Token::RBracket) {
            let name = match self.current() {
                Some(Token::Symbol(s)) => s.clone(),
                _ => return Err(fmt_msg(MsgKey::NeedsSymbol, &["let"]).to_string()),
            };
            self.advance();

            let value = self.parse_expr()?;
            bindings.push((name, value));
        }

        self.expect(Token::RBracket)?;

        // 本体のパース
        let body = Box::new(self.parse_expr()?);
        self.expect(Token::RParen)?;

        Ok(Expr::Let { bindings, body })
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.advance(); // 'if'をスキップ

        let test = Box::new(self.parse_expr()?);
        let then = Box::new(self.parse_expr()?);

        let otherwise = if self.current() != Some(&Token::RParen) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.expect(Token::RParen)?;

        Ok(Expr::If {
            test,
            then,
            otherwise,
        })
    }

    fn parse_do(&mut self) -> Result<Expr, String> {
        self.advance(); // 'do'をスキップ

        let mut exprs = Vec::new();
        while self.current() != Some(&Token::RParen) {
            exprs.push(self.parse_expr()?);
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Do(exprs))
    }

    fn parse_vector(&mut self) -> Result<Expr, String> {
        self.expect(Token::LBracket)?;

        let mut items = Vec::new();
        while self.current() != Some(&Token::RBracket) {
            items.push(self.parse_expr()?);
        }

        self.expect(Token::RBracket)?;

        Ok(Expr::Vector(items))
    }

    fn parse_map(&mut self) -> Result<Expr, String> {
        self.expect(Token::LBrace)?;

        let mut pairs = Vec::new();
        while self.current() != Some(&Token::RBrace) {
            let key = self.parse_expr()?;
            let value = self.parse_expr()?;
            pairs.push((key, value));
        }

        self.expect(Token::RBrace)?;

        Ok(Expr::Map(pairs))
    }

    fn parse_quote(&mut self) -> Result<Expr, String> {
        self.advance(); // 'をスキップ
        let expr = self.parse_expr()?;
        Ok(Expr::Call {
            func: Box::new(Expr::Symbol("quote".to_string())),
            args: vec![expr],
        })
    }

    fn parse_match(&mut self) -> Result<Expr, String> {
        self.advance(); // 'match'をスキップ

        // マッチする式
        let expr = Box::new(self.parse_expr()?);

        // アームのパース
        let mut arms = Vec::new();
        while self.current() != Some(&Token::RParen) {
            let pattern = self.parse_pattern()?;

            // ガード条件のチェック
            let guard = if self.current() == Some(&Token::When) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            // ->
            self.expect(Token::Arrow)?;

            // 結果式
            let body = Box::new(self.parse_expr()?);

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Match { expr, arms })
    }

    /// (try expr)
    fn parse_try(&mut self) -> Result<Expr, String> {
        self.advance(); // 'try'をスキップ

        let expr = Box::new(self.parse_expr()?);

        self.expect(Token::RParen)?;

        Ok(Expr::Try(expr))
    }

    /// (defer expr)
    fn parse_defer(&mut self) -> Result<Expr, String> {
        self.advance(); // 'defer'をスキップ

        let expr = Box::new(self.parse_expr()?);

        self.expect(Token::RParen)?;

        Ok(Expr::Defer(expr))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match self.current() {
            Some(Token::Symbol(s)) if s == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Some(Token::Nil) => {
                self.advance();
                Ok(Pattern::Nil)
            }
            Some(Token::True) => {
                self.advance();
                Ok(Pattern::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Pattern::Bool(false))
            }
            Some(Token::Integer(n)) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Integer(n))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Ok(Pattern::Float(f))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::String(s))
            }
            Some(Token::Keyword(k)) => {
                let k = k.clone();
                self.advance();
                Ok(Pattern::Keyword(k))
            }
            Some(Token::Symbol(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Var(s))
            }
            Some(Token::LBracket) => self.parse_vector_pattern(),
            Some(Token::LBrace) => self.parse_map_pattern(),
            Some(token) => Err(fmt_msg(MsgKey::UnexpectedPattern, &[&format!("{:?}", token)])),
            None => Err(msg(MsgKey::UnexpectedEof).to_string()),
        }
    }

    fn parse_vector_pattern(&mut self) -> Result<Pattern, String> {
        self.expect(Token::LBracket)?;

        let mut patterns = Vec::new();
        while self.current() != Some(&Token::RBracket) {
            patterns.push(self.parse_pattern()?);
        }

        self.expect(Token::RBracket)?;

        Ok(Pattern::Vector(patterns))
    }

    fn parse_map_pattern(&mut self) -> Result<Pattern, String> {
        self.expect(Token::LBrace)?;

        let mut pairs = Vec::new();
        while self.current() != Some(&Token::RBrace) {
            let key = match self.current() {
                Some(Token::Keyword(k)) => k.clone(),
                _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
            };
            self.advance();

            let pattern = self.parse_pattern()?;
            pairs.push((key, pattern));
        }

        self.expect(Token::RBrace)?;

        Ok(Pattern::Map(pairs))
    }

    /// (module name)
    fn parse_module(&mut self) -> Result<Expr, String> {
        self.advance(); // 'module'をスキップ

        let name = match self.current() {
            Some(Token::Symbol(n)) => n.clone(),
            _ => return Err(msg(MsgKey::ModuleNeedsName).to_string()),
        };
        self.advance();

        self.expect(Token::RParen)?;

        Ok(Expr::Module(name))
    }

    /// (export sym1 sym2 ...)
    fn parse_export(&mut self) -> Result<Expr, String> {
        self.advance(); // 'export'をスキップ

        let mut symbols = Vec::new();
        while self.current() != Some(&Token::RParen) {
            match self.current() {
                Some(Token::Symbol(s)) => {
                    symbols.push(s.clone());
                    self.advance();
                }
                _ => return Err(msg(MsgKey::ExportNeedsSymbols).to_string()),
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Export(symbols))
    }

    /// (use module :only [syms] | :as alias | :all)
    fn parse_use(&mut self) -> Result<Expr, String> {
        self.advance(); // 'use'をスキップ

        // モジュール名
        let module = match self.current() {
            Some(Token::Symbol(n)) => n.clone(),
            _ => return Err(msg(MsgKey::UseNeedsModuleName).to_string()),
        };
        self.advance();

        // モード指定
        let mode = match self.current() {
            Some(Token::Keyword(k)) if k == "only" => {
                self.advance();
                // [sym1 sym2 ...]
                self.expect(Token::LBracket)?;
                let mut symbols = Vec::new();
                while self.current() != Some(&Token::RBracket) {
                    match self.current() {
                        Some(Token::Symbol(s)) => {
                            symbols.push(s.clone());
                            self.advance();
                        }
                        _ => return Err(msg(MsgKey::ExpectedSymbolInOnlyList).to_string()),
                    }
                }
                self.expect(Token::RBracket)?;
                UseMode::Only(symbols)
            }
            Some(Token::Keyword(k)) if k == "as" => {
                self.advance();
                match self.current() {
                    Some(Token::Symbol(alias)) => {
                        let alias = alias.clone();
                        self.advance();
                        UseMode::As(alias)
                    }
                    _ => return Err(msg(MsgKey::AsNeedsAlias).to_string()),
                }
            }
            Some(Token::Keyword(k)) if k == "all" => {
                self.advance();
                UseMode::All
            }
            _ => return Err(msg(MsgKey::UseNeedsMode).to_string()),
        };

        self.expect(Token::RParen)?;

        Ok(Expr::Use { module, mode })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let mut parser = Parser::new("42").unwrap();
        assert_eq!(parser.parse().unwrap(), Expr::Integer(42));
    }

    #[test]
    fn test_parse_symbol() {
        let mut parser = Parser::new("foo").unwrap();
        assert_eq!(parser.parse().unwrap(), Expr::Symbol("foo".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let mut parser = Parser::new("(+ 1 2)").unwrap();
        match parser.parse().unwrap() {
            Expr::Call { func, args } => {
                assert_eq!(*func, Expr::Symbol("+".to_string()));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_parse_def() {
        let mut parser = Parser::new("(def x 42)").unwrap();
        match parser.parse().unwrap() {
            Expr::Def(name, value) => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expr::Integer(42));
            }
            _ => panic!("Expected Def"),
        }
    }

    #[test]
    fn test_parse_fn() {
        let mut parser = Parser::new("(fn [x y] (+ x y))").unwrap();
        match parser.parse().unwrap() {
            Expr::Fn { params, .. } => {
                assert_eq!(params, vec!["x", "y"]);
            }
            _ => panic!("Expected Fn"),
        }
    }

    #[test]
    fn test_parse_module() {
        let mut parser = Parser::new("(module http)").unwrap();
        match parser.parse().unwrap() {
            Expr::Module(name) => {
                assert_eq!(name, "http");
            }
            _ => panic!("Expected Module"),
        }
    }

    #[test]
    fn test_parse_export() {
        let mut parser = Parser::new("(export get post)").unwrap();
        match parser.parse().unwrap() {
            Expr::Export(symbols) => {
                assert_eq!(symbols, vec!["get", "post"]);
            }
            _ => panic!("Expected Export"),
        }
    }

    #[test]
    fn test_parse_use_only() {
        let mut parser = Parser::new("(use http :only [get post])").unwrap();
        match parser.parse().unwrap() {
            Expr::Use { module, mode } => {
                assert_eq!(module, "http");
                assert_eq!(mode, UseMode::Only(vec!["get".to_string(), "post".to_string()]));
            }
            _ => panic!("Expected Use"),
        }
    }

    #[test]
    fn test_parse_use_as() {
        let mut parser = Parser::new("(use http :as h)").unwrap();
        match parser.parse().unwrap() {
            Expr::Use { module, mode } => {
                assert_eq!(module, "http");
                assert_eq!(mode, UseMode::As("h".to_string()));
            }
            _ => panic!("Expected Use"),
        }
    }

    #[test]
    fn test_parse_use_all() {
        let mut parser = Parser::new("(use http :all)").unwrap();
        match parser.parse().unwrap() {
            Expr::Use { module, mode } => {
                assert_eq!(module, "http");
                assert_eq!(mode, UseMode::All);
            }
            _ => panic!("Expected Use"),
        }
    }

    #[test]
    fn test_parse_try() {
        let mut parser = Parser::new("(try (/ 1 0))").unwrap();
        match parser.parse().unwrap() {
            Expr::Try(_) => {}
            _ => panic!("Expected Try"),
        }
    }

    #[test]
    fn test_parse_defer() {
        let mut parser = Parser::new("(defer (print \"cleanup\"))").unwrap();
        match parser.parse().unwrap() {
            Expr::Defer(_) => {}
            _ => panic!("Expected Defer"),
        }
    }
}
