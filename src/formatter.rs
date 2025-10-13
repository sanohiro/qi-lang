//! Qiコードフォーマッター
//!
//! STYLE_GUIDE.mdに基づいてQiコードを整形します。

use crate::value::{Expr, FStringPart, FnParam, MatchArm, Pattern, UseMode};

/// フォーマット設定
#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub indent_width: usize,
    pub max_line_length: usize,
    pub match_arrow_spacing: ArrowSpacing,
}

/// matchアローのスペース設定
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowSpacing {
    Both,   // pattern -> action
    Before, // pattern-> action
    After,  // pattern ->action
    None,   // pattern->action
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            indent_width: 2,
            max_line_length: 100,
            match_arrow_spacing: ArrowSpacing::Both,
        }
    }
}

/// 式をフォーマットする（エントリーポイント）
pub fn format_expr(expr: &Expr) -> String {
    let config = FormatConfig::default();
    format_expr_with_config(expr, &config, 0)
}

/// 式をフォーマットする（設定とインデント指定）
fn format_expr_with_config(expr: &Expr, config: &FormatConfig, indent: usize) -> String {
    match expr {
        // リテラル
        Expr::Nil => "nil".to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Integer(n) => n.to_string(),
        Expr::Float(f) => format_float(*f),
        Expr::String(s) => format_string(s),
        Expr::FString(parts) => format_fstring(parts, config, indent),
        Expr::Symbol(s) => s.clone(),
        Expr::Keyword(k) => format!(":{}", k),

        // コレクション
        Expr::List(items) => format_list(items, config, indent),
        Expr::Vector(items) => format_vector(items, config, indent),
        Expr::Map(pairs) => format_map(pairs, config, indent),

        // 特殊形式
        Expr::Def(name, value, is_private) => format_def(name, value, *is_private, config, indent),
        Expr::Fn {
            params,
            body,
            is_variadic,
        } => format_fn(params, body, *is_variadic, config, indent),
        Expr::Let { bindings, body } => format_let(bindings, body, config, indent),
        Expr::If {
            test,
            then,
            otherwise,
        } => format_if(test, then, otherwise, config, indent),
        Expr::Do(exprs) => format_do(exprs, config, indent),
        Expr::Match { expr, arms } => format_match(expr, arms, config, indent),
        Expr::Try(expr) => format_try(expr, config, indent),
        Expr::Defer(expr) => format_defer(expr, config, indent),
        Expr::Loop { bindings, body } => format_loop(bindings, body, config, indent),
        Expr::Recur(args) => format_recur(args, config, indent),

        // マクロ
        Expr::Mac {
            name,
            params,
            is_variadic,
            body,
        } => format_mac(name, params, *is_variadic, body, config, indent),
        Expr::Quasiquote(expr) => format!(
            "(quasiquote {})",
            format_expr_with_config(expr, config, indent)
        ),
        Expr::Unquote(expr) => format!(
            "(unquote {})",
            format_expr_with_config(expr, config, indent)
        ),
        Expr::UnquoteSplice(expr) => format!(
            "(unquote-splice {})",
            format_expr_with_config(expr, config, indent)
        ),

        // モジュール
        Expr::Module(name) => format!("(module {})", name),
        Expr::Export(names) => format_export(names),
        Expr::Use { module, mode } => format_use(module, mode),

        // 関数呼び出し
        Expr::Call { func, args } => format_call(func, args, config, indent),
    }
}

/// 浮動小数点数をフォーマット
fn format_float(f: f64) -> String {
    // 整数値の場合は.0を付ける
    if f.fract() == 0.0 && f.is_finite() {
        format!("{:.1}", f)
    } else {
        f.to_string()
    }
}

/// 文字列をフォーマット（エスケープ処理）
fn format_string(s: &str) -> String {
    // 常にエスケープシーケンスを使用
    // 複数行文字列（"""）は使わず、\n等でエスケープする
    format!("\"{}\"", escape_string(s))
}

/// 文字列をエスケープ
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('"', "\\\"")
}

/// F-stringをフォーマット
fn format_fstring(parts: &[FStringPart], _config: &FormatConfig, _indent: usize) -> String {
    let mut result = String::from("f\"");
    for part in parts {
        match part {
            FStringPart::Text(text) => result.push_str(&escape_string(text)),
            FStringPart::Code(code) => {
                result.push('{');
                result.push_str(code);
                result.push('}');
            }
        }
    }
    result.push('"');
    result
}

/// リストをフォーマット
fn format_list(items: &[Expr], config: &FormatConfig, indent: usize) -> String {
    if items.is_empty() {
        return "()".to_string();
    }

    let formatted_items: Vec<String> = items
        .iter()
        .map(|e| format_expr_with_config(e, config, indent))
        .collect();

    let one_line = format!("({})", formatted_items.join(" "));
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        // 複数行で整形
        let indent_str = " ".repeat(indent + config.indent_width);
        let items_str = formatted_items.join(&format!("\n{}", indent_str));
        format!("({}\n{})", items_str, " ".repeat(indent))
    }
}

/// ベクタをフォーマット
fn format_vector(items: &[Expr], config: &FormatConfig, indent: usize) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }

    let formatted_items: Vec<String> = items
        .iter()
        .map(|e| format_expr_with_config(e, config, indent))
        .collect();

    let one_line = format!("[{}]", formatted_items.join(" "));
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        // 複数行で整形
        let indent_str = " ".repeat(indent + config.indent_width);
        let items_str = formatted_items
            .iter()
            .map(|s| format!("{}{}", indent_str, s))
            .collect::<Vec<_>>()
            .join("\n");
        format!("[\n{}\n{}]", items_str, " ".repeat(indent))
    }
}

/// マップをフォーマット
fn format_map(pairs: &[(Expr, Expr)], config: &FormatConfig, indent: usize) -> String {
    if pairs.is_empty() {
        return "{}".to_string();
    }

    let formatted_pairs: Vec<String> = pairs
        .iter()
        .map(|(k, v)| {
            format!(
                "{} {}",
                format_expr_with_config(k, config, indent),
                format_expr_with_config(v, config, indent)
            )
        })
        .collect();

    let one_line = format!("{{{}}}", formatted_pairs.join(" "));
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        // 複数行で整形
        let indent_str = " ".repeat(indent + config.indent_width);
        let pairs_str = formatted_pairs
            .iter()
            .map(|s| format!("{}{}", indent_str, s))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{{\n{}\n{}}}", pairs_str, " ".repeat(indent))
    }
}

/// defをフォーマット
fn format_def(
    name: &str,
    value: &Expr,
    is_private: bool,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let keyword = if is_private { "def-" } else { "def" };
    let value_str = format_expr_with_config(value, config, indent + config.indent_width);

    let one_line = format!("({} {} {})", keyword, name, value_str);
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        // 複数行で整形
        let indent_str = " ".repeat(indent + config.indent_width);
        format!("({} {}\n{}{})", keyword, name, indent_str, value_str)
    }
}

/// fnをフォーマット
fn format_fn(
    params: &[FnParam],
    body: &Expr,
    is_variadic: bool,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let params_str = format_fn_params(params, is_variadic);
    let body_str = format_expr_with_config(body, config, indent + config.indent_width);

    let one_line = format!("(fn {} {})", params_str, body_str);
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        let indent_str = " ".repeat(indent + config.indent_width);
        format!("(fn {}\n{}{})", params_str, indent_str, body_str)
    }
}

/// 関数パラメータをフォーマット
fn format_fn_params(params: &[FnParam], is_variadic: bool) -> String {
    let param_strs: Vec<String> = params.iter().map(format_fn_param).collect();

    if is_variadic {
        if param_strs.is_empty() {
            "[& args]".to_string()
        } else {
            format!("[{} & args]", param_strs.join(" "))
        }
    } else {
        format!("[{}]", param_strs.join(" "))
    }
}

/// 単一の関数パラメータをフォーマット
fn format_fn_param(param: &FnParam) -> String {
    match param {
        FnParam::Simple(s) => s.clone(),
        FnParam::Vector(params, rest) => {
            let params_str: Vec<String> = params.iter().map(format_fn_param).collect();
            if let Some(rest_param) = rest {
                format!(
                    "[{} ...{}]",
                    params_str.join(" "),
                    format_fn_param(rest_param)
                )
            } else {
                format!("[{}]", params_str.join(" "))
            }
        }
        FnParam::Map(pairs, as_var) => {
            let pairs_str: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!(":{} {}", k, format_fn_param(v)))
                .collect();
            if let Some(as_name) = as_var {
                format!("{{{}:as {}}}", pairs_str.join(" "), as_name)
            } else {
                format!("{{{}}}", pairs_str.join(" "))
            }
        }
    }
}

/// letをフォーマット
fn format_let(
    bindings: &[(FnParam, Expr)],
    body: &Expr,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let indent_str = " ".repeat(indent + config.indent_width);
    let binding_indent = indent + config.indent_width + 1; // "[" の分

    let bindings_str: Vec<String> = bindings
        .iter()
        .map(|(param, expr)| {
            format!(
                "{}{} {}",
                indent_str,
                format_fn_param(param),
                format_expr_with_config(expr, config, binding_indent)
            )
        })
        .collect();

    let body_str = format_expr_with_config(body, config, indent + config.indent_width);

    format!(
        "(let [\n{}\n{}]\n{}{})",
        bindings_str.join("\n"),
        indent_str.trim_end(),
        indent_str,
        body_str
    )
}

/// ifをフォーマット
fn format_if(
    test: &Expr,
    then: &Expr,
    otherwise: &Option<Box<Expr>>,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let test_str = format_expr_with_config(test, config, indent + config.indent_width);
    let then_str = format_expr_with_config(then, config, indent + config.indent_width);
    let indent_str = " ".repeat(indent + config.indent_width);

    if let Some(else_expr) = otherwise {
        let else_str = format_expr_with_config(else_expr, config, indent + config.indent_width);
        format!(
            "(if {}\n{}{}\n{}{})",
            test_str, indent_str, then_str, indent_str, else_str
        )
    } else {
        format!("(if {}\n{}{})", test_str, indent_str, then_str)
    }
}

/// doをフォーマット
fn format_do(exprs: &[Expr], config: &FormatConfig, indent: usize) -> String {
    if exprs.is_empty() {
        return "(do)".to_string();
    }

    let indent_str = " ".repeat(indent + config.indent_width);
    let exprs_str: Vec<String> = exprs
        .iter()
        .map(|e| {
            format!(
                "{}{}",
                indent_str,
                format_expr_with_config(e, config, indent + config.indent_width)
            )
        })
        .collect();

    format!("(do\n{})", exprs_str.join("\n"))
}

/// matchをフォーマット
fn format_match(expr: &Expr, arms: &[MatchArm], config: &FormatConfig, indent: usize) -> String {
    let expr_str = format_expr_with_config(expr, config, indent + config.indent_width);
    let indent_str = " ".repeat(indent + config.indent_width);

    let arms_str: Vec<String> = arms
        .iter()
        .map(|arm| {
            let pattern_str = format_pattern(&arm.pattern);
            let arrow = match config.match_arrow_spacing {
                ArrowSpacing::Both => " -> ",
                ArrowSpacing::Before => " ->",
                ArrowSpacing::After => "-> ",
                ArrowSpacing::None => "->",
            };
            let body_str = format_expr_with_config(&arm.body, config, indent + config.indent_width);

            if let Some(guard) = &arm.guard {
                let guard_str =
                    format_expr_with_config(guard, config, indent + config.indent_width);
                format!(
                    "{}{} when {}{}{}",
                    indent_str, pattern_str, guard_str, arrow, body_str
                )
            } else {
                format!("{}{}{}{}", indent_str, pattern_str, arrow, body_str)
            }
        })
        .collect();

    format!("(match {}\n{})", expr_str, arms_str.join("\n"))
}

/// パターンをフォーマット
fn format_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Nil => "nil".to_string(),
        Pattern::Bool(b) => b.to_string(),
        Pattern::Integer(n) => n.to_string(),
        Pattern::Float(f) => format_float(*f),
        Pattern::String(s) => format_string(s),
        Pattern::Keyword(k) => format!(":{}", k),
        Pattern::Var(v) => v.clone(),
        Pattern::List(items, rest) => {
            let items_str: Vec<String> = items.iter().map(format_pattern).collect();
            if let Some(rest_pattern) = rest {
                format!(
                    "[{} ...{}]",
                    items_str.join(" "),
                    format_pattern(rest_pattern)
                )
            } else {
                format!("[{}]", items_str.join(" "))
            }
        }
        Pattern::Vector(items) => {
            let items_str: Vec<String> = items.iter().map(format_pattern).collect();
            format!("[{}]", items_str.join(" "))
        }
        Pattern::Map(pairs) => {
            let pairs_str: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!(":{} {}", k, format_pattern(v)))
                .collect();
            format!("{{{}}}", pairs_str.join(" "))
        }
        Pattern::As(pattern, var) => format!("{} :as {}", format_pattern(pattern), var),
        Pattern::Transform(var, _expr) => {
            // Transformパターンは複雑なので簡略化
            format!("{} => ...", var)
        }
        Pattern::Or(patterns) => {
            let patterns_str: Vec<String> = patterns.iter().map(format_pattern).collect();
            patterns_str.join(" | ")
        }
    }
}

/// tryをフォーマット
fn format_try(expr: &Expr, config: &FormatConfig, indent: usize) -> String {
    let expr_str = format_expr_with_config(expr, config, indent + config.indent_width);
    let indent_str = " ".repeat(indent + config.indent_width);
    format!("(try\n{}{})", indent_str, expr_str)
}

/// deferをフォーマット
fn format_defer(expr: &Expr, config: &FormatConfig, indent: usize) -> String {
    let expr_str = format_expr_with_config(expr, config, indent);
    format!("(defer {})", expr_str)
}

/// loopをフォーマット
fn format_loop(
    bindings: &[(String, Expr)],
    body: &Expr,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let indent_str = " ".repeat(indent + config.indent_width);
    let binding_indent = indent + config.indent_width + 1;

    let bindings_str: Vec<String> = bindings
        .iter()
        .map(|(name, expr)| {
            format!(
                "{}{} {}",
                indent_str,
                name,
                format_expr_with_config(expr, config, binding_indent)
            )
        })
        .collect();

    let body_str = format_expr_with_config(body, config, indent + config.indent_width);

    format!(
        "(loop [\n{}\n{}]\n{}{})",
        bindings_str.join("\n"),
        indent_str.trim_end(),
        indent_str,
        body_str
    )
}

/// recurをフォーマット
fn format_recur(args: &[Expr], config: &FormatConfig, indent: usize) -> String {
    let args_str: Vec<String> = args
        .iter()
        .map(|e| format_expr_with_config(e, config, indent))
        .collect();
    format!("(recur {})", args_str.join(" "))
}

/// macをフォーマット
fn format_mac(
    name: &str,
    params: &[String],
    is_variadic: bool,
    body: &Expr,
    config: &FormatConfig,
    indent: usize,
) -> String {
    let params_str = if is_variadic {
        if params.is_empty() {
            "[& args]".to_string()
        } else {
            format!("[{} & args]", params.join(" "))
        }
    } else {
        format!("[{}]", params.join(" "))
    };

    let body_str = format_expr_with_config(body, config, indent + config.indent_width);
    let indent_str = " ".repeat(indent + config.indent_width);

    format!("(mac {} {}\n{}{})", name, params_str, indent_str, body_str)
}

/// exportをフォーマット
fn format_export(names: &[String]) -> String {
    if names.len() <= 3 {
        format!("(export [{}])", names.join(" "))
    } else {
        let indent_str = "  ";
        let names_str = names
            .iter()
            .map(|n| format!("{}{}", indent_str, n))
            .collect::<Vec<_>>()
            .join("\n");
        format!("(export\n  [\n{}\n  ])", names_str)
    }
}

/// useをフォーマット
fn format_use(module: &str, mode: &UseMode) -> String {
    match mode {
        UseMode::All => format!("(use {} :all)", module),
        UseMode::As(alias) => format!("(use {} :as {})", module, alias),
        UseMode::Only(names) => {
            if names.len() <= 3 {
                format!("(use {} :only [{}])", module, names.join(" "))
            } else {
                let indent_str = "  ";
                let names_str = names.join(&format!("\n{}", indent_str));
                format!("(use {}\n  :only [\n{}{}])", module, indent_str, names_str)
            }
        }
    }
}

/// 関数呼び出しをフォーマット
fn format_call(func: &Expr, args: &[Expr], config: &FormatConfig, indent: usize) -> String {
    let func_str = format_expr_with_config(func, config, indent);

    if args.is_empty() {
        return format!("({})", func_str);
    }

    let args_str: Vec<String> = args
        .iter()
        .map(|e| format_expr_with_config(e, config, indent + config.indent_width))
        .collect();

    let one_line = format!("({} {})", func_str, args_str.join(" "));
    if one_line.len() <= config.max_line_length {
        one_line
    } else {
        // 複数行で整形
        let indent_str = " ".repeat(indent + config.indent_width);
        let args_formatted = args_str
            .iter()
            .map(|s| format!("{}{}", indent_str, s))
            .collect::<Vec<_>>()
            .join("\n");
        format!("({}\n{})", func_str, args_formatted)
    }
}

/// 複数の式をフォーマット（トップレベル用）
pub fn format_exprs(exprs: &[Expr]) -> String {
    exprs
        .iter()
        .map(format_expr)
        .collect::<Vec<_>>()
        .join("\n\n") // トップレベル定義間は1行空行
}
