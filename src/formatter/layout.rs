//! Docツリーを最終レイアウトへ変換するフォーマットエンジン。
//!
//! `tokenizer` と `doc` が保持するトリビア情報を用いて Qi ソースコードを整形する。
//! 構文を変更した場合はこのモジュールと `tokenizer.rs` / `doc.rs` を必ず同期させること。

use super::doc::{DocExpr, DocExprKind, DocNode, TriviaKind};
use super::tokenizer::{tokenize, CommentKind};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::Path;

static CONFIG_PAIR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r":([A-Za-z0-9_-]+)\s+([0-9]+)").expect("valid regex"));

/// フォーマッタの設定値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutConfig {
    pub indent_width: usize,
    pub blank_lines_between_defs: usize,
    pub max_line_length: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            indent_width: 2,
            blank_lines_between_defs: 1,
            max_line_length: 100,
        }
    }
}

/// `.qi-format.edn` を読み込み設定を返す。失敗時はデフォルトを返す。
pub fn load_config(path: Option<&str>) -> LayoutConfig {
    if let Some(path) = path {
        if let Some(cfg) = read_config(Path::new(path)) {
            return cfg;
        }
    }

    let default_path = Path::new(".qi-format.edn");
    if let Some(cfg) = read_config(default_path) {
        return cfg;
    }

    LayoutConfig::default()
}

fn read_config(path: &Path) -> Option<LayoutConfig> {
    let content = fs::read_to_string(path).ok()?;
    Some(parse_config(&content))
}

fn parse_config(content: &str) -> LayoutConfig {
    let mut cfg = LayoutConfig::default();
    for caps in CONFIG_PAIR_RE.captures_iter(content) {
        let key = &caps[1];
        if let Ok(value) = caps[2].parse::<usize>() {
            match key {
                "indent-width" => cfg.indent_width = value,
                "blank-lines-between-defs" => cfg.blank_lines_between_defs = value,
                "max-line-length" => cfg.max_line_length = value,
                _ => {}
            }
        }
    }
    cfg
}

/// ソースコードを Doc ツリー経由でフォーマットする。
pub fn format_source(source: &str, config: &LayoutConfig) -> Result<String, String> {
    let tokens = tokenize(source)?;
    let doc_nodes = super::doc::parse_tokens(&tokens).map_err(|e| e.to_string())?;
    let mut renderer = Renderer::new(config.clone());
    renderer.render_module(&doc_nodes);
    Ok(renderer.finish())
}

#[derive(Debug, Clone)]
struct Renderer {
    cfg: LayoutConfig,
    lines: Vec<String>,
}

impl Renderer {
    fn new(cfg: LayoutConfig) -> Self {
        Self {
            cfg,
            lines: Vec::new(),
        }
    }

    fn finish(mut self) -> String {
        while let Some(true) = self.lines.last().map(|l| l.trim().is_empty()) {
            self.lines.pop();
        }
        self.lines.push(String::new());
        self.lines.join("\n")
    }

    fn render_module(&mut self, nodes: &[DocNode]) {
        let ModuleCollection {
            items: module_items,
            trailing_comments,
        } = collect_module_items(nodes);
        let mut first = true;
        let mut last_was_def = false;

        let mut previous_had_expr = false;
        for item in module_items {
            if !first {
                let required = if last_was_def && is_def_form(&item.expr) {
                    self.cfg.blank_lines_between_defs.max(1)
                } else {
                    1
                };
                let blanks = item.blank_lines_before.max(required);
                self.push_blank_lines(blanks);
            } else if item.blank_lines_before > 0 {
                self.push_blank_lines(item.blank_lines_before);
            }

            let mut comments = item.comments.clone();
            if !comments.is_empty()
                && comments[0].starts_with(';')
                && !comments[0].starts_with(";;")
                && previous_had_expr
            {
                if let Some(last_line) = self.lines.last_mut() {
                    last_line.push_str("  ");
                    last_line.push_str(&comments[0]);
                    comments.remove(0);
                }
            }

            for comment in comments {
                self.lines.push(comment);
            }

            let mut expr_lines = self.render_expr_lines(&item.expr, 0);
            if let Some(comment) = &item.inline_comment {
                if let Some(last) = expr_lines.last_mut() {
                    last.push_str("  ");
                    last.push_str(comment);
                }
            }
            self.lines.extend(expr_lines);

            last_was_def = is_def_form(&item.expr);
            first = false;
            previous_had_expr = true;
        }

        if !trailing_comments.is_empty() {
            if !self.lines.is_empty() {
                self.push_blank_lines(1);
            }
            for comment in trailing_comments {
                self.lines.push(comment);
            }
        }
    }

    fn render_expr_lines(&self, expr: &DocExpr, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut pending_blank = 0;

        for trivia in &expr.leading {
            match trivia.kind {
                TriviaKind::Whitespace => {
                    pending_blank = pending_blank.max(count_extra_blank_lines(&trivia.text));
                }
                TriviaKind::Comment(_) => {
                    for _ in 0..pending_blank {
                        lines.push(String::new());
                    }
                    pending_blank = 0;
                    lines.push(format!("{}{}", spaces(indent), trim_comment(&trivia.text)));
                }
            }
        }
        for _ in 0..pending_blank {
            lines.push(String::new());
        }

        match &expr.kind {
            DocExprKind::Atom { text, .. } => {
                lines.push(format!("{}{}", spaces(indent), text));
            }
            DocExprKind::List(items) => {
                if let Some(inline) = self.inline_list(expr, items) {
                    lines.push(format!("{}{}", spaces(indent), inline));
                } else {
                    lines.extend(self.render_sequence(items, indent, '(', ')'));
                }
            }
            DocExprKind::Vector(items) => {
                if let Some(inline) = self.inline_vector(expr, items) {
                    lines.push(format!("{}{}", spaces(indent), inline));
                } else {
                    lines.extend(self.render_sequence(items, indent, '[', ']'));
                }
            }
            DocExprKind::Map(items) => {
                if let Some(inline) = self.inline_map(expr, items) {
                    lines.push(format!("{}{}", spaces(indent), inline));
                } else {
                    lines.extend(self.render_sequence(items, indent, '{', '}'));
                }
            }
        }

        lines
    }

    fn render_sequence(
        &self,
        nodes: &[DocNode],
        indent: usize,
        open: char,
        close: char,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let indent_str = spaces(indent);
        let child_indent = indent + self.cfg.indent_width;

        let items = collect_sequence_items(nodes);
        if items.is_empty() {
            lines.push(format!("{}{}{}", indent_str, open, close));
            return lines;
        }

        let head_sym: Option<String> = head_symbol(nodes);
        let mut idx = 0usize;

        while idx < items.len() && items[idx].expr.is_none() {
            let item = &items[idx];
            for _ in 0..item.blank_lines_before {
                lines.push(String::new());
            }
            for comment in &item.leading_comments {
                lines.push(format!("{}{}", spaces(child_indent), comment));
            }
            idx += 1;
        }

        let mut first_line = format!("{}{}", indent_str, open);
        let inline_start = idx;
        if open == '[' {
            if let Some(binding_lines) = self.render_binding_vector_layout(&items, indent) {
                return binding_lines;
            }
        }

        let mut inline_parts: Vec<String> = Vec::new();
        let mut inline_len = first_line.len();
        let mut trailing_comment: Option<String> = None;
        let inline_limit = if open == '[' { 2 } else { usize::MAX };

        while idx < items.len() {
            let item = &items[idx];
            if item.expr.is_none()
                || item.blank_lines_before > 0
                || !item.leading_comments.is_empty()
            {
                break;
            }
            if let Some(expr) = &item.expr {
                if let Some(inline) = self.inline_expr(expr) {
                    if let Some(ref head) = head_sym {
                        let limit = match head.as_str() {
                            "defn" | "defn-" => 3,
                            _ => inline_limit,
                        };
                        if inline_parts.len() >= limit {
                            break;
                        }
                    } else if inline_parts.len() >= inline_limit {
                        break;
                    }
                    let extra = if inline_len == first_line.len() {
                        inline.len()
                    } else {
                        inline.len() + 1
                    };
                    if inline_len + extra <= self.cfg.max_line_length {
                        if let Some(comment) = &item.inline_comment {
                            trailing_comment = Some(comment.clone());
                        }
                        inline_parts.push(inline.clone());
                        inline_len += extra;
                        idx += 1;
                        continue;
                    }
                }
            }
            break;
        }

        if !inline_parts.is_empty() {
            for (i, part) in inline_parts.iter().enumerate() {
                if first_line.len() > indent_str.len() + 1 || i > 0 {
                    first_line.push(' ');
                }
                first_line.push_str(part);
            }
        }

        lines.push(first_line);

        let mut remaining_start = if inline_parts.is_empty() {
            inline_start
        } else {
            idx
        };

        while remaining_start < items.len() {
            let item = &items[remaining_start];
            for _ in 0..item.blank_lines_before {
                lines.push(String::new());
            }
            for comment in &item.leading_comments {
                lines.push(format!("{}{}", spaces(child_indent), comment));
            }
            if let Some(expr) = &item.expr {
                let mut expr_lines = self.render_expr_lines(expr, child_indent);
                if let Some(comment) = &item.inline_comment {
                    if let Some(last) = expr_lines.last_mut() {
                        last.push(' ');
                        last.push_str(comment);
                    }
                }
                lines.extend(expr_lines);
            }
            remaining_start += 1;
        }

        let mut appended_close = false;
        if remaining_start > inline_start {
            if let Some(pos) = lines
                .iter_mut()
                .rposition(|line| !line.trim().is_empty() && !line.trim_start().starts_with(';'))
            {
                lines[pos].push(close);
                if let Some(comment) = &trailing_comment {
                    lines[pos].push_str("  ");
                    lines[pos].push_str(comment);
                }
                appended_close = true;
            }
        } else if let Some(line) = lines.last_mut() {
            line.push(close);
            if let Some(comment) = &trailing_comment {
                line.push_str("  ");
                line.push_str(comment);
            }
            appended_close = true;
        }

        if !appended_close {
            lines.push(format!("{}{}", indent_str, close));
        }

        lines
    }

    fn push_blank_lines(&mut self, count: usize) {
        for _ in 0..count {
            self.lines.push(String::new());
        }
    }

    fn inline_expr(&self, expr: &DocExpr) -> Option<String> {
        if expr
            .leading
            .iter()
            .any(|t| matches!(t.kind, TriviaKind::Comment(_)))
        {
            return None;
        }

        match &expr.kind {
            DocExprKind::Atom { text, .. } => Some(text.clone()),
            DocExprKind::List(items) => self.inline_sequence(expr, items, '(', ')'),
            DocExprKind::Vector(items) => self.inline_sequence(expr, items, '[', ']'),
            DocExprKind::Map(items) => self.inline_sequence(expr, items, '{', '}'),
        }
    }

    fn inline_list(&self, expr: &DocExpr, nodes: &[DocNode]) -> Option<String> {
        if expr
            .leading
            .iter()
            .any(|t| matches!(t.kind, TriviaKind::Comment(_)))
        {
            return None;
        }
        if let Some(head) = head_symbol(nodes) {
            if matches!(head.as_str(), "defn" | "defn-") {
                return None;
            }
            if head == "let" {
                let expr_count = nodes
                    .iter()
                    .filter(|n| matches!(n, DocNode::Expr(_)))
                    .count();
                if expr_count > 2 {
                    return None;
                }
            }
        }
        self.inline_sequence(expr, nodes, '(', ')')
    }

    fn inline_vector(&self, expr: &DocExpr, nodes: &[DocNode]) -> Option<String> {
        if expr
            .leading
            .iter()
            .any(|t| matches!(t.kind, TriviaKind::Comment(_)))
        {
            return None;
        }
        let expr_count = nodes
            .iter()
            .filter(|n| matches!(n, DocNode::Expr(_)))
            .count();
        if expr_count > 2 {
            return None;
        }
        self.inline_sequence(expr, nodes, '[', ']')
    }

    fn inline_map(&self, expr: &DocExpr, nodes: &[DocNode]) -> Option<String> {
        if expr
            .leading
            .iter()
            .any(|t| matches!(t.kind, TriviaKind::Comment(_)))
        {
            return None;
        }
        self.inline_sequence(expr, nodes, '{', '}')
    }

    fn inline_sequence(
        &self,
        _expr: &DocExpr,
        nodes: &[DocNode],
        open: char,
        close: char,
    ) -> Option<String> {
        let mut parts = Vec::new();
        for node in nodes {
            match node {
                DocNode::Trivia(trivia) => match trivia.kind {
                    TriviaKind::Whitespace => continue,
                    TriviaKind::Comment(_) => return None,
                },
                DocNode::Expr(expr) => {
                    let part = self.inline_expr(expr)?;
                    parts.push(part);
                }
            }
        }

        let mut result = String::new();
        result.push(open);
        result.push_str(&parts.join(" "));
        result.push(close);

        if result.len() <= self.cfg.max_line_length {
            Some(result)
        } else {
            None
        }
    }

    fn render_binding_vector_layout(
        &self,
        items: &[SequenceItem],
        indent: usize,
    ) -> Option<Vec<String>> {
        if items.len() < 4 || items.len() % 2 != 0 {
            return None;
        }
        let mut parts = Vec::new();
        for item in items {
            if item.blank_lines_before > 0
                || !item.leading_comments.is_empty()
                || item.inline_comment.is_some()
            {
                return None;
            }
            let expr = item.expr.as_ref()?;
            let inline = self.inline_expr(expr)?;
            parts.push(inline);
        }
        let mut lines = Vec::new();
        let base_indent = spaces(indent);
        let pair_indent = spaces(indent + self.cfg.indent_width * 3);

        let mut iter = parts.chunks(2);
        if let Some(first_pair) = iter.next() {
            let line = format!("{}[{} {}", base_indent, first_pair[0], first_pair[1]);
            lines.push(line);
        }
        for chunk in iter {
            let line = format!("{}{} {}", pair_indent, chunk[0], chunk[1]);
            lines.push(line);
        }
        if let Some(last) = lines.last_mut() {
            last.push(']');
        }
        Some(lines)
    }
}

#[derive(Debug, Clone)]
struct ModuleItem {
    pub blank_lines_before: usize,
    pub comments: Vec<String>,
    pub expr: DocExpr,
    pub inline_comment: Option<String>,
}

#[derive(Debug, Clone)]
struct SequenceItem {
    pub blank_lines_before: usize,
    pub leading_comments: Vec<String>,
    pub expr: Option<DocExpr>,
    pub inline_comment: Option<String>,
}

struct ModuleCollection {
    items: Vec<ModuleItem>,
    trailing_comments: Vec<String>,
}

fn collect_module_items(nodes: &[DocNode]) -> ModuleCollection {
    let mut items: Vec<ModuleItem> = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut pending_blank = 0usize;
    let mut trailing_comments = Vec::new();

    for node in nodes {
        match node {
            DocNode::Trivia(trivia) => match trivia.kind {
                TriviaKind::Whitespace => {
                    pending_blank = pending_blank.max(count_extra_blank_lines(&trivia.text));
                }
                TriviaKind::Comment(kind) => match kind {
                    CommentKind::Inline => {
                        if let Some(last) = items.last_mut() {
                            last.inline_comment = Some(trim_comment(&trivia.text));
                        } else {
                            pending_comments.push(trim_comment(&trivia.text));
                        }
                    }
                    CommentKind::Line => {
                        if pending_blank == 0 && pending_comments.is_empty() {
                            if let Some(last) = items.last_mut() {
                                if last.inline_comment.is_none() {
                                    last.inline_comment = Some(trim_comment(&trivia.text));
                                    continue;
                                }
                            }
                        }
                        pending_comments.push(trim_comment(&trivia.text));
                    }
                },
            },
            DocNode::Expr(expr) => {
                items.push(ModuleItem {
                    blank_lines_before: pending_blank,
                    comments: std::mem::take(&mut pending_comments),
                    expr: expr.clone(),
                    inline_comment: None,
                });
                pending_blank = 0;
            }
        }
    }

    if !pending_comments.is_empty() {
        for _ in 0..pending_blank {
            trailing_comments.push(String::new());
        }
        trailing_comments.extend(pending_comments);
    }

    ModuleCollection {
        items,
        trailing_comments,
    }
}

fn collect_sequence_items(nodes: &[DocNode]) -> Vec<SequenceItem> {
    let mut items: Vec<SequenceItem> = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut pending_blank = 0usize;

    for node in nodes {
        match node {
            DocNode::Trivia(trivia) => match trivia.kind {
                TriviaKind::Whitespace => {
                    pending_blank = pending_blank.max(count_extra_blank_lines(&trivia.text));
                }
                TriviaKind::Comment(kind) => match kind {
                    CommentKind::Line => {
                        pending_comments.push(trim_comment(&trivia.text));
                    }
                    CommentKind::Inline => {
                        if let Some(last) = items.last_mut() {
                            last.inline_comment = Some(trim_comment(&trivia.text));
                        } else {
                            pending_comments.push(trim_comment(&trivia.text));
                        }
                    }
                },
            },
            DocNode::Expr(expr) => {
                items.push(SequenceItem {
                    blank_lines_before: pending_blank,
                    leading_comments: std::mem::take(&mut pending_comments),
                    expr: Some(expr.clone()),
                    inline_comment: None,
                });
                pending_blank = 0;
            }
        }
    }

    if !pending_comments.is_empty() {
        items.push(SequenceItem {
            blank_lines_before: pending_blank,
            leading_comments: pending_comments,
            expr: None,
            inline_comment: None,
        });
    }

    items
}

fn count_extra_blank_lines(text: &str) -> usize {
    let newline_count = text.chars().filter(|&c| c == '\n').count();
    newline_count.saturating_sub(1)
}

fn trim_comment(text: &str) -> String {
    text.trim_end_matches(['\r', '\n']).to_string()
}

fn head_symbol(nodes: &[DocNode]) -> Option<String> {
    for node in nodes {
        match node {
            DocNode::Trivia(_) => continue,
            DocNode::Expr(expr) => {
                if expr
                    .leading
                    .iter()
                    .any(|t| matches!(t.kind, TriviaKind::Comment(_)))
                {
                    return None;
                }
                if let DocExprKind::Atom { text, .. } = &expr.kind {
                    return Some(text.clone());
                }
                return None;
            }
        }
    }
    None
}

fn spaces(indent: usize) -> String {
    " ".repeat(indent)
}

fn is_def_form(expr: &DocExpr) -> bool {
    match &expr.kind {
        DocExprKind::List(items) => {
            for node in items {
                match node {
                    DocNode::Trivia(_) => continue,
                    DocNode::Expr(head) => {
                        if let DocExprKind::Atom { text, .. } = &head.kind {
                            return matches!(
                                text.as_str(),
                                "def" | "defn" | "defn-" | "module" | "export" | "use"
                            );
                        }
                        return false;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let cfg = parse_config("{:indent-width 4 :blank-lines-between-defs 2}");
        assert_eq!(cfg.indent_width, 4);
        assert_eq!(cfg.blank_lines_between_defs, 2);
        assert_eq!(cfg.max_line_length, 100);
    }

    #[test]
    fn round_trip_identity() {
        let src = "(defn greet [name]\n  (println \"hi\" name))\n";
        let formatted = format_source(src, &LayoutConfig::default()).unwrap();
        assert_eq!(formatted, src);
    }

    #[test]
    fn formats_nested_list() {
        let src = "(let [a 1\n      b 2]\n  (do\n    (println a)\n    (println b)))\n";
        let expected = "(let [a 1 b 2] (do (println a) (println b)))\n";
        let formatted = format_source(src, &LayoutConfig::default()).unwrap();
        assert_eq!(formatted, expected);
    }

    #[test]
    fn preserves_comments_and_spacing() {
        let src = ";; header\n(def x 10)  ; inline\n(def y 20)\n";
        let expected = ";; header\n(def x 10)\n\n; inline\n(def y 20)\n";
        let formatted = format_source(src, &LayoutConfig::default()).unwrap();
        assert_eq!(formatted, expected);
    }
}
