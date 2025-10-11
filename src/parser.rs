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
        loop {
            match self.current() {
                Some(Token::Pipe) => {
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
                Some(Token::PipeRailway) => {
                    self.advance();
                    let right = self.parse_primary()?;

                    // x |>? f を (_railway-pipe f x) に変換
                    expr = Expr::Call {
                        func: Box::new(Expr::Symbol("_railway-pipe".to_string())),
                        args: vec![right, expr],
                    };
                }
                Some(Token::ParallelPipe) => {
                    self.advance();
                    let right = self.parse_primary()?;

                    // x ||> f を (pmap f x) に変換
                    expr = Expr::Call {
                        func: Box::new(Expr::Symbol("pmap".to_string())),
                        args: vec![right, expr],
                    };
                }
                _ => break,
            }
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
            Some(Token::Backquote) => self.parse_quasiquote(),
            Some(Token::Unquote) => self.parse_unquote(),
            Some(Token::UnquoteSplice) => self.parse_unquote_splice(),
            Some(Token::At) => self.parse_at(),
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
                "loop" => return self.parse_loop(),
                "recur" => return self.parse_recur(),
                "mac" => return self.parse_mac(),
                "flow" => return self.parse_flow(),
                "module" => return self.parse_module(),
                "export" => return self.parse_export(),
                "use" => return self.parse_use(),
                _ => {}
            }
        }

        // 通常のリスト（関数呼び出し）
        let first_expr = self.parse_primary()?;  // パイプラインを含まない

        // パイプラインのチェック
        if self.current() == Some(&Token::Pipe)
            || self.current() == Some(&Token::PipeRailway)
            || self.current() == Some(&Token::ParallelPipe)
            || self.current() == Some(&Token::AsyncPipe) {
            let mut expr = first_expr;
            let mut is_async = false;

            loop {
                match self.current() {
                    Some(Token::Pipe) => {
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
                    Some(Token::PipeRailway) => {
                        self.advance();
                        let right = self.parse_primary()?;

                        // x |>? f を (_railway-pipe f x) に変換
                        expr = Expr::Call {
                            func: Box::new(Expr::Symbol("_railway-pipe".to_string())),
                            args: vec![right, expr],
                        };
                    }
                    Some(Token::ParallelPipe) => {
                        self.advance();
                        let right = self.parse_primary()?;

                        // x ||> f を (pmap f x) に変換
                        expr = Expr::Call {
                            func: Box::new(Expr::Symbol("pmap".to_string())),
                            args: vec![right, expr],
                        };
                    }
                    Some(Token::AsyncPipe) => {
                        is_async = true;
                        self.advance();
                        let right = self.parse_primary()?;

                        // パイプラインとして処理（通常のPipeと同じ）
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
                    _ => break,
                }
            }
            self.expect(Token::RParen)?;

            // ~> が使われていたら、全体を (go (fn [] ...)) でラップ
            if is_async {
                // (fn [] expr) を作成
                let lambda = Expr::Fn {
                    params: vec![],
                    body: Box::new(expr),
                    is_variadic: false,
                };

                // (go lambda) を作成
                expr = Expr::Call {
                    func: Box::new(Expr::Symbol("go".to_string())),
                    args: vec![lambda],
                };
            }

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
            let guard = if matches!(self.current(), Some(Token::Symbol(s)) if s == "when") {
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

    fn parse_loop(&mut self) -> Result<Expr, String> {
        self.advance(); // 'loop'をスキップ

        // 束縛のパース [var1 val1 var2 val2 ...]
        self.expect(Token::LBracket)?;
        let mut bindings = Vec::new();

        while self.current() != Some(&Token::RBracket) {
            let name = match self.current() {
                Some(Token::Symbol(s)) => s.clone(),
                _ => return Err(fmt_msg(MsgKey::NeedsSymbol, &["loop"]).to_string()),
            };
            self.advance();

            let value = self.parse_expr()?;
            bindings.push((name, value));
        }

        self.expect(Token::RBracket)?;

        // 本体のパース
        let body = Box::new(self.parse_expr()?);
        self.expect(Token::RParen)?;

        Ok(Expr::Loop { bindings, body })
    }

    fn parse_recur(&mut self) -> Result<Expr, String> {
        self.advance(); // 'recur'をスキップ

        let mut args = Vec::new();
        while self.current() != Some(&Token::RParen) {
            args.push(self.parse_expr()?);
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Recur(args))
    }

    /// macをパース
    fn parse_mac(&mut self) -> Result<Expr, String> {
        self.advance(); // 'mac'をスキップ

        // マクロ名
        let name = match self.current() {
            Some(Token::Symbol(s)) => s.clone(),
            _ => return Err(fmt_msg(MsgKey::NeedsSymbol, &["mac"]).to_string()),
        };
        self.advance();

        // パラメータリスト
        self.expect(Token::LBracket)?;
        let mut params = Vec::new();
        let mut is_variadic = false;

        while self.current() != Some(&Token::RBracket) {
            match self.current() {
                Some(Token::Symbol(s)) if s == "&" => {
                    is_variadic = true;
                    self.advance();
                    // 次のシンボルが可変引数名
                    match self.current() {
                        Some(Token::Symbol(s)) => {
                            params.push(s.clone());
                            self.advance();
                        }
                        _ => return Err(msg(MsgKey::MacVarargNeedsSymbol).to_string()),
                    }
                }
                Some(Token::Symbol(s)) => {
                    params.push(s.clone());
                    self.advance();
                }
                _ => return Err(fmt_msg(MsgKey::NeedsSymbol, &["mac"]).to_string()),
            }
        }

        self.expect(Token::RBracket)?;

        // 本体
        let body = Box::new(self.parse_expr()?);

        self.expect(Token::RParen)?;

        Ok(Expr::Mac {
            name,
            params,
            is_variadic,
            body,
        })
    }

    /// flowをパース
    /// (flow data |> fn1 |> fn2) → (data |> fn1 |> fn2)
    /// (flow |> fn1 |> fn2) → (fn [x] (x |> fn1 |> fn2))
    fn parse_flow(&mut self) -> Result<Expr, String> {
        self.advance(); // 'flow'をスキップ

        // 最初が|>の場合、ラムダを生成
        if self.current() == Some(&Token::Pipe) {
            // (flow |> fn1 |> fn2) → (fn [__flow_x] (__flow_x |> fn1 |> fn2))
            let var_name = "__flow_x".to_string();
            let mut expr = Expr::Symbol(var_name.clone());

            // パイプラインをパース
            while self.current() == Some(&Token::Pipe)
                || self.current() == Some(&Token::PipeRailway)
                || self.current() == Some(&Token::ParallelPipe) {
                match self.current() {
                    Some(Token::Pipe) => {
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
                    Some(Token::PipeRailway) => {
                        self.advance();
                        let right = self.parse_primary()?;
                        expr = Expr::Call {
                            func: Box::new(Expr::Symbol("_railway-pipe".to_string())),
                            args: vec![right, expr],
                        };
                    }
                    Some(Token::ParallelPipe) => {
                        self.advance();
                        let right = self.parse_primary()?;
                        expr = Expr::Call {
                            func: Box::new(Expr::Symbol("pmap".to_string())),
                            args: vec![right, expr],
                        };
                    }
                    _ => break,
                }
            }

            self.expect(Token::RParen)?;

            // ラムダでラップ
            Ok(Expr::Fn {
                params: vec![var_name],
                body: Box::new(expr),
                is_variadic: false,
            })
        } else {
            // (flow data |> fn1 |> fn2) → 通常のパイプラインとしてパース
            let first = self.parse_primary()?;

            // パイプラインのチェック
            if self.current() == Some(&Token::Pipe)
                || self.current() == Some(&Token::PipeRailway)
                || self.current() == Some(&Token::ParallelPipe) {
                let mut expr = first;

                // パイプラインをパース
                while self.current() == Some(&Token::Pipe)
                    || self.current() == Some(&Token::PipeRailway)
                    || self.current() == Some(&Token::ParallelPipe) {
                    match self.current() {
                        Some(Token::Pipe) => {
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
                        Some(Token::PipeRailway) => {
                            self.advance();
                            let right = self.parse_primary()?;
                            expr = Expr::Call {
                                func: Box::new(Expr::Symbol("_railway-pipe".to_string())),
                                args: vec![right, expr],
                            };
                        }
                        Some(Token::ParallelPipe) => {
                            self.advance();
                            let right = self.parse_primary()?;
                            expr = Expr::Call {
                                func: Box::new(Expr::Symbol("pmap".to_string())),
                                args: vec![right, expr],
                            };
                        }
                        _ => break,
                    }
                }

                self.expect(Token::RParen)?;
                Ok(expr)
            } else {
                // パイプラインなし、単にfirstを返す
                self.expect(Token::RParen)?;
                Ok(first)
            }
        }
    }

    /// quasiquoteをパース
    fn parse_quasiquote(&mut self) -> Result<Expr, String> {
        self.advance(); // `をスキップ
        let expr = Box::new(self.parse_expr()?);
        Ok(Expr::Quasiquote(expr))
    }

    /// unquoteをパース
    fn parse_unquote(&mut self) -> Result<Expr, String> {
        self.advance(); // ,をスキップ
        let expr = Box::new(self.parse_expr()?);
        Ok(Expr::Unquote(expr))
    }

    /// unquote-spliceをパース
    fn parse_unquote_splice(&mut self) -> Result<Expr, String> {
        self.advance(); // ,@をスキップ
        let expr = Box::new(self.parse_expr()?);
        Ok(Expr::UnquoteSplice(expr))
    }

    /// @構文: @expr => (deref expr)
    fn parse_at(&mut self) -> Result<Expr, String> {
        self.advance(); // @をスキップ
        let expr = self.parse_primary()?;
        Ok(Expr::Call {
            func: Box::new(Expr::Symbol("deref".to_string())),
            args: vec![expr],
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        // 最初のパターンをパース
        let first_pattern = self.parse_single_pattern()?;

        // | があればorパターンとして複数パターンを収集
        if self.current() == Some(&Token::Bar) {
            let mut patterns = vec![first_pattern];

            while self.current() == Some(&Token::Bar) {
                self.advance(); // |
                patterns.push(self.parse_single_pattern()?);
            }

            Ok(Pattern::Or(patterns))
        } else {
            Ok(first_pattern)
        }
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern, String> {
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
        let mut as_var = None;

        while self.current() != Some(&Token::RBrace) {
            // :as チェック
            if let Some(Token::Keyword(k)) = self.current() {
                if k == "as" {
                    self.advance(); // :as
                    // 次は変数名
                    match self.current() {
                        Some(Token::Symbol(var)) => {
                            as_var = Some(var.clone());
                            self.advance();
                            break;
                        }
                        _ => return Err(msg(MsgKey::AsNeedsVarName).to_string()),
                    }
                }
            }

            let key = match self.current() {
                Some(Token::Keyword(k)) => k.clone(),
                _ => return Err(fmt_msg(MsgKey::KeyMustBeKeyword, &[])),
            };
            self.advance();

            // 変数名またはパターン
            let pattern = if let Some(Token::Symbol(var)) = self.current() {
                let var_name = var.clone();
                self.advance();

                // => チェック
                if self.current() == Some(&Token::FatArrow) {
                    // {:key var => transform} の形式
                    self.advance(); // =>
                    let transform = self.parse_primary()?;
                    Pattern::Transform(var_name, Box::new(transform))
                } else {
                    // 通常の変数パターン
                    Pattern::Var(var_name)
                }
            } else {
                self.parse_pattern()?
            };

            pairs.push((key, pattern));
        }

        self.expect(Token::RBrace)?;

        let map_pattern = Pattern::Map(pairs);

        // :as があれば As パターンでラップ
        if let Some(var) = as_var {
            Ok(Pattern::As(Box::new(map_pattern), var))
        } else {
            Ok(map_pattern)
        }
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
