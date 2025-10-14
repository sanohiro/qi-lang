//! フォーマッタ用のドキュメントツリー構築モジュール。
//!
//! 既存の `parser.rs` をそのまま流用するとコメントやホワイトスペース（トリビア）が失われるため、
//! フォーマット専用に S 式を読み直してトリビア付きの木構造を生成する。
//!
//! Qi の構文を変更する際は、`parser.rs` と本モジュール、さらに `tokenizer.rs` の
//! それぞれを忘れずに同期させること。

use super::tokenizer::{CommentKind, FmtToken, FmtTokenKind};
use std::fmt;

/// トリビアの種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    Comment(CommentKind),
}

/// コメントや空白などのトリビア。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub text: String,
}

impl Trivia {
    pub fn whitespace(text: String) -> Self {
        Trivia {
            kind: TriviaKind::Whitespace,
            text,
        }
    }

    pub fn comment(kind: CommentKind, text: String) -> Self {
        Trivia {
            kind: TriviaKind::Comment(kind),
            text,
        }
    }
}

/// 原始要素の種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomKind {
    Symbol,
    Keyword,
    Number,
    StringLiteral,
    FStringLiteral,
    Other,
}

/// ドキュメントツリー上のノード。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocNode {
    Expr(DocExpr),
    Trivia(Trivia),
}

/// ドキュメントツリー上の式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExpr {
    pub leading: Vec<Trivia>,
    pub kind: DocExprKind,
}

/// 式の具体的な内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocExprKind {
    List(Vec<DocNode>),
    Vector(Vec<DocNode>),
    Map(Vec<DocNode>),
    Atom { kind: AtomKind, text: String },
}

/// ドキュメントツリーの構築時に発生するエラー。
#[derive(Debug, Clone)]
pub enum DocParseError {
    UnexpectedToken {
        expected: &'static str,
        found: Option<FmtTokenKind>,
    },
    UnexpectedEof(&'static str),
}

impl fmt::Display for DocParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocParseError::UnexpectedToken { expected, found } => match found {
                Some(kind) => write!(f, "expected {}, but found {:?}", expected, kind),
                None => write!(f, "expected {}, but reached EOF", expected),
            },
            DocParseError::UnexpectedEof(ctx) => write!(f, "unexpected EOF while parsing {}", ctx),
        }
    }
}

impl std::error::Error for DocParseError {}

/// トークン列からドキュメントツリーを構築する。
pub fn parse_tokens(tokens: &[FmtToken]) -> Result<Vec<DocNode>, DocParseError> {
    Parser::new(tokens).parse_all()
}

struct Parser<'a> {
    tokens: &'a [FmtToken],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [FmtToken]) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_all(mut self) -> Result<Vec<DocNode>, DocParseError> {
        let mut nodes = Vec::new();
        loop {
            let trivia = self.collect_trivia();
            if self.peek_kind().is_none() {
                for t in trivia {
                    nodes.push(DocNode::Trivia(t));
                }
                break;
            }

            let expr = self.parse_expr_with_leading(trivia)?;
            nodes.push(DocNode::Expr(expr));
        }
        Ok(nodes)
    }

    fn parse_expr_with_leading(&mut self, leading: Vec<Trivia>) -> Result<DocExpr, DocParseError> {
        match self.peek_kind() {
            Some(FmtTokenKind::LParen) => {
                self.index += 1;
                let items = self.parse_container(FmtTokenKind::RParen, "list")?;
                Ok(DocExpr {
                    leading,
                    kind: DocExprKind::List(items),
                })
            }
            Some(FmtTokenKind::LBracket) => {
                self.index += 1;
                let items = self.parse_container(FmtTokenKind::RBracket, "vector")?;
                Ok(DocExpr {
                    leading,
                    kind: DocExprKind::Vector(items),
                })
            }
            Some(FmtTokenKind::LBrace) => {
                self.index += 1;
                let items = self.parse_container(FmtTokenKind::RBrace, "map")?;
                Ok(DocExpr {
                    leading,
                    kind: DocExprKind::Map(items),
                })
            }
            Some(
                kind @ (FmtTokenKind::Symbol
                | FmtTokenKind::Keyword
                | FmtTokenKind::Number
                | FmtTokenKind::StringLiteral
                | FmtTokenKind::FStringLiteral
                | FmtTokenKind::Other),
            ) => {
                let token = self.advance().unwrap();
                let atom_kind = match kind {
                    FmtTokenKind::Symbol => AtomKind::Symbol,
                    FmtTokenKind::Keyword => AtomKind::Keyword,
                    FmtTokenKind::Number => AtomKind::Number,
                    FmtTokenKind::StringLiteral => AtomKind::StringLiteral,
                    FmtTokenKind::FStringLiteral => AtomKind::FStringLiteral,
                    _ => AtomKind::Other,
                };
                Ok(DocExpr {
                    leading,
                    kind: DocExprKind::Atom {
                        kind: atom_kind,
                        text: token.lexeme.clone(),
                    },
                })
            }
            other => Err(DocParseError::UnexpectedToken {
                expected: "expression",
                found: other,
            }),
        }
    }

    fn parse_container(
        &mut self,
        closing: FmtTokenKind,
        ctx: &'static str,
    ) -> Result<Vec<DocNode>, DocParseError> {
        let mut items = Vec::new();
        loop {
            let trivia = self.collect_trivia();
            if self.peek_kind() == Some(closing.clone()) {
                self.index += 1;
                for t in trivia {
                    items.push(DocNode::Trivia(t));
                }
                break;
            }
            if self.peek_kind().is_none() {
                return Err(DocParseError::UnexpectedEof(ctx));
            }

            let expr = self.parse_expr_with_leading(trivia)?;
            items.push(DocNode::Expr(expr));
        }
        Ok(items)
    }

    fn collect_trivia(&mut self) -> Vec<Trivia> {
        let mut trivia = Vec::new();
        while let Some(kind) = self.peek_kind() {
            match kind {
                FmtTokenKind::Whitespace => {
                    let token = self.advance().unwrap();
                    trivia.push(Trivia::whitespace(token.lexeme.clone()));
                }
                FmtTokenKind::Comment(comment_kind) => {
                    let token = self.advance().unwrap();
                    trivia.push(Trivia::comment(comment_kind, token.lexeme.clone()));
                }
                _ => break,
            }
        }
        trivia
    }

    fn peek_kind(&self) -> Option<FmtTokenKind> {
        self.tokens.get(self.index).map(|t| t.kind.clone())
    }

    fn advance(&mut self) -> Option<&FmtToken> {
        let token = self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::tokenizer::{tokenize, CommentKind};
    use crate::i18n;

    #[test]
    fn parse_simple_list_with_comments() {
        i18n::init();
        let src = ";; top\n(def x 10)\n";
        let tokens = tokenize(src).unwrap();
        let nodes = parse_tokens(&tokens).unwrap();
        let exprs: Vec<&DocExpr> = nodes
            .iter()
            .filter_map(|n| match n {
                DocNode::Expr(e) => Some(e),
                _ => None,
            })
            .collect();
        assert_eq!(exprs.len(), 1);
        let expr = exprs[0];
        assert_eq!(expr.leading.len(), 1);
        assert!(expr
            .leading
            .iter()
            .any(|t| matches!(t.kind, TriviaKind::Comment(CommentKind::Line))));
        match &expr.kind {
            DocExprKind::List(items) => assert!(!items.is_empty()),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn parse_vector_with_inline_comment() {
        i18n::init();
        let src = "[1 ; inline\n 2]\n";
        let tokens = tokenize(src).unwrap();
        let nodes = parse_tokens(&tokens).unwrap();
        match &nodes[0] {
            DocNode::Expr(expr) => match &expr.kind {
                DocExprKind::Vector(items) => {
                    assert!(items.len() >= 2);
                    if let DocNode::Expr(second) = &items[1] {
                        assert!(second
                            .leading
                            .iter()
                            .any(|t| matches!(t.kind, TriviaKind::Comment(CommentKind::Inline))));
                    } else {
                        panic!("expected expression for second element");
                    }
                }
                _ => panic!("expected vector"),
            },
            _ => panic!("expected expr"),
        }
    }

    #[test]
    fn parse_nested_map() {
        i18n::init();
        let src = "{:a 1 :b {:c 2}}\n";
        let tokens = tokenize(src).unwrap();
        let nodes = parse_tokens(&tokens).unwrap();
        let exprs: Vec<&DocExpr> = nodes
            .iter()
            .filter_map(|n| match n {
                DocNode::Expr(e) => Some(e),
                _ => None,
            })
            .collect();
        assert_eq!(exprs.len(), 1);
        let expr = exprs[0];
        match &expr.kind {
            DocExprKind::Map(items) => assert!(!items.is_empty()),
            _ => panic!("expected map expr"),
        }
    }
}
