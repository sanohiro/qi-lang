//! Markdown生成・加工関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// header - Markdownヘッダーを生成
/// 引数: (level text) - レベル (1-6)、テキスト
/// 例: (markdown/header 2 "Report") → "## Report"
pub fn native_markdown_header(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/header", "2"]));
    }

    let level = match &args[0] {
        Value::Integer(n) if *n >= 1 && *n <= 6 => *n as usize,
        Value::Integer(_) => {
            return Err(fmt_msg(
                MsgKey::MdHeaderInvalidLevel,
                &[&args[0].to_string()],
            ))
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/header", "an integer"],
            ))
        }
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["markdown/header", "a string"],
            ))
        }
    };

    let header = format!("{} {}", "#".repeat(level), text);
    Ok(Value::String(header))
}

/// list - Markdown箇条書きリストを生成
/// 引数: (items) - 文字列のリストまたはベクタ
/// 例: (markdown/list ["A" "B" "C"]) → "- A\n- B\n- C"
pub fn native_markdown_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/list", "1"]));
    }

    let items = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/list", "a list or vector"],
            ))
        }
    };

    let mut lines = Vec::new();
    for item in items {
        let text = match item {
            Value::String(s) => s.clone(),
            _ => item.to_string(),
        };
        lines.push(format!("- {}", text));
    }

    Ok(Value::String(lines.join("\n")))
}

/// ordered-list - Markdown番号付きリストを生成
/// 引数: (items) - 文字列のリストまたはベクタ
/// 例: (markdown/ordered-list ["First" "Second"]) → "1. First\n2. Second"
pub fn native_markdown_ordered_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["markdown/ordered-list", "1"],
        ));
    }

    let items = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/ordered-list", "a list or vector"],
            ))
        }
    };

    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let text = match item {
            Value::String(s) => s.clone(),
            _ => item.to_string(),
        };
        lines.push(format!("{}. {}", i + 1, text));
    }

    Ok(Value::String(lines.join("\n")))
}

/// table - Markdownテーブルを生成
/// 引数: (rows) - 行のリストまたはベクタ（最初の行がヘッダー）
/// 例: (markdown/table [["Name" "Score"] ["Alice" "95"] ["Bob" "87"]])
pub fn native_markdown_table(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/table", "1"]));
    }

    let rows = match &args[0] {
        Value::List(rows) | Value::Vector(rows) => rows,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/table", "a list or vector"],
            ))
        }
    };

    if rows.is_empty() {
        return Err(fmt_msg(MsgKey::MdTableEmpty, &[]));
    }

    // ヘッダー行を処理
    let header_row = match &rows[0] {
        Value::List(cells) | Value::Vector(cells) => cells,
        _ => return Err(fmt_msg(MsgKey::MdTableRowMustBeList, &["0"])),
    };

    if header_row.is_empty() {
        return Err(fmt_msg(MsgKey::MdTableEmpty, &[]));
    }

    let col_count = header_row.len();
    let mut lines = Vec::new();

    // ヘッダー
    let header_cells: Vec<String> = header_row
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        })
        .collect();
    lines.push(format!("| {} |", header_cells.join(" | ")));

    // セパレーター
    lines.push(format!("| {} |", vec!["---"; col_count].join(" | ")));

    // データ行
    for (i, row) in rows.iter().skip(1).enumerate() {
        let cells = match row {
            Value::List(cells) | Value::Vector(cells) => cells,
            _ => {
                return Err(fmt_msg(
                    MsgKey::MdTableRowMustBeList,
                    &[&(i + 1).to_string()],
                ))
            }
        };

        if cells.len() != col_count {
            return Err(fmt_msg(
                MsgKey::MdTableColumnMismatch,
                &[
                    &(i + 1).to_string(),
                    &cells.len().to_string(),
                    &col_count.to_string(),
                ],
            ));
        }

        let cell_texts: Vec<String> = cells
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            })
            .collect();
        lines.push(format!("| {} |", cell_texts.join(" | ")));
    }

    Ok(Value::String(lines.join("\n")))
}

/// code-block - Markdownコードブロックを生成
/// 引数: (lang code) - 言語名、コード文字列
/// 例: (markdown/code-block "qi" "(+ 1 2)") → "```qi\n(+ 1 2)\n```"
pub fn native_markdown_code_block(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["markdown/code-block", "2"],
        ));
    }

    let lang = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Nil => String::new(),
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/code-block", "a string"],
            ))
        }
    };

    let code = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["markdown/code-block", "a string"],
            ))
        }
    };

    let block = if lang.is_empty() {
        format!("```\n{}\n```", code)
    } else {
        format!("```{}\n{}\n```", lang, code)
    };

    Ok(Value::String(block))
}

/// join - 複数のMarkdown要素を改行で結合
/// 引数: (parts) - 文字列のリストまたはベクタ
/// 例: (markdown/join ["# Title" "Content"]) → "# Title\n\nContent"
pub fn native_markdown_join(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/join", "1"]));
    }

    let parts = match &args[0] {
        Value::List(parts) | Value::Vector(parts) => parts,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/join", "a list or vector"],
            ))
        }
    };

    let text_parts: Vec<String> = parts
        .iter()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        })
        .collect();

    Ok(Value::String(text_parts.join("\n\n")))
}

/// link - Markdownリンクを生成
/// 引数: (text url) - リンクテキスト、URL
/// 例: (markdown/link "GitHub" "https://github.com") → "[GitHub](https://github.com)"
pub fn native_markdown_link(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/link", "2"]));
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/link", "a string"],
            ))
        }
    };

    let url = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["markdown/link", "a string"],
            ))
        }
    };

    Ok(Value::String(format!("[{}]({})", text, url)))
}

/// image - Markdown画像記法を生成
/// 引数: (alt src) - 代替テキスト、画像パス/URL
/// 例: (markdown/image "Logo" "logo.png") → "![Logo](logo.png)"
pub fn native_markdown_image(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/image", "2"]));
    }

    let alt = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/image", "a string"],
            ))
        }
    };

    let src = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["markdown/image", "a string"],
            ))
        }
    };

    Ok(Value::String(format!("![{}]({})", alt, src)))
}

// 正規表現パターン（遅延初期化）
static CODE_BLOCK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```([^\n]*)\n([\s\S]*?)```").unwrap());

static HEADER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(#{1,6})\s+(.+)$").unwrap());

static LIST_ITEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[-*+]\s+(.+)$").unwrap());

static ORDERED_LIST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+\.\s+(.+)$").unwrap());

/// extract-code-blocks - Markdownからコードブロックを抽出
/// 引数: (text) - Markdown文字列
/// 戻り値: [{:lang "qi" :code "(+ 1 2)"} ...] のリスト
/// 例: (markdown/extract-code-blocks doc) → コードブロックのリスト
pub fn native_markdown_extract_code_blocks(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["markdown/extract-code-blocks", "1"],
        ));
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/extract-code-blocks", "a string"],
            ))
        }
    };

    let mut blocks = Vec::new();

    for cap in CODE_BLOCK_REGEX.captures_iter(text) {
        let lang = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let code = cap.get(2).map(|m| m.as_str().trim_end()).unwrap_or("");

        let mut block = HashMap::new();
        block.insert(
            "lang".to_string(),
            if lang.is_empty() {
                Value::Nil
            } else {
                Value::String(lang.to_string())
            },
        );
        block.insert("code".to_string(), Value::String(code.to_string()));

        blocks.push(Value::Map(block));
    }

    Ok(Value::List(blocks))
}

/// parse - Markdown文字列をASTに変換
/// 引数: (text) - Markdown文字列
/// 戻り値: ブロック要素のリスト
/// 例: (markdown/parse "# Title\n\nHello") → [{:type "header" :level 1 :text "Title"} {:type "paragraph" :text "Hello"}]
pub fn native_markdown_parse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["markdown/parse", "1"]));
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/parse", "a string"],
            ))
        }
    };

    let mut blocks = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // 空行をスキップ
        if line.is_empty() {
            i += 1;
            continue;
        }

        // コードブロック
        if line.starts_with("```") {
            let lang = line.trim_start_matches("```").trim();
            let mut code_lines = Vec::new();
            i += 1;

            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }

            let mut block = HashMap::new();
            block.insert("type".to_string(), Value::String("code-block".to_string()));
            block.insert(
                "lang".to_string(),
                if lang.is_empty() {
                    Value::Nil
                } else {
                    Value::String(lang.to_string())
                },
            );
            block.insert("code".to_string(), Value::String(code_lines.join("\n")));
            blocks.push(Value::Map(block));
            i += 1;
            continue;
        }

        // ヘッダー
        if let Some(cap) = HEADER_REGEX.captures(line) {
            let level = cap.get(1).map(|m| m.as_str().len()).unwrap_or(1);
            let text = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut block = HashMap::new();
            block.insert("type".to_string(), Value::String("header".to_string()));
            block.insert("level".to_string(), Value::Integer(level as i64));
            block.insert("text".to_string(), Value::String(text.to_string()));
            blocks.push(Value::Map(block));
            i += 1;
            continue;
        }

        // リスト（順不同）
        if LIST_ITEM_REGEX.is_match(line) {
            let mut items = Vec::new();
            while i < lines.len() {
                let current = lines[i].trim();
                if let Some(cap) = LIST_ITEM_REGEX.captures(current) {
                    let item = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    items.push(Value::String(item.to_string()));
                    i += 1;
                } else {
                    break;
                }
            }

            let mut block = HashMap::new();
            block.insert("type".to_string(), Value::String("list".to_string()));
            block.insert("ordered".to_string(), Value::Bool(false));
            block.insert("items".to_string(), Value::List(items));
            blocks.push(Value::Map(block));
            continue;
        }

        // リスト（番号付き）
        if ORDERED_LIST_REGEX.is_match(line) {
            let mut items = Vec::new();
            while i < lines.len() {
                let current = lines[i].trim();
                if let Some(cap) = ORDERED_LIST_REGEX.captures(current) {
                    let item = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    items.push(Value::String(item.to_string()));
                    i += 1;
                } else {
                    break;
                }
            }

            let mut block = HashMap::new();
            block.insert("type".to_string(), Value::String("list".to_string()));
            block.insert("ordered".to_string(), Value::Bool(true));
            block.insert("items".to_string(), Value::List(items));
            blocks.push(Value::Map(block));
            continue;
        }

        // 段落（デフォルト）
        let mut para_lines = Vec::new();
        while i < lines.len() {
            let current = lines[i].trim();
            if current.is_empty()
                || current.starts_with('#')
                || current.starts_with("```")
                || LIST_ITEM_REGEX.is_match(current)
                || ORDERED_LIST_REGEX.is_match(current)
            {
                break;
            }
            para_lines.push(current);
            i += 1;
        }

        if !para_lines.is_empty() {
            let mut block = HashMap::new();
            block.insert("type".to_string(), Value::String("paragraph".to_string()));
            block.insert("text".to_string(), Value::String(para_lines.join(" ")));
            blocks.push(Value::Map(block));
        }
    }

    Ok(Value::List(blocks))
}

/// stringify - ASTをMarkdown文字列に変換
/// 引数: (blocks) - ブロック要素のリスト
/// 戻り値: Markdown文字列
/// 例: (markdown/stringify ast) → Markdown文字列
pub fn native_markdown_stringify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedExactlyNArgs,
            &["markdown/stringify", "1"],
        ));
    }

    let blocks = match &args[0] {
        Value::List(blocks) | Value::Vector(blocks) => blocks,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["markdown/stringify", "a list or vector"],
            ))
        }
    };

    let mut result = Vec::new();

    for block in blocks {
        let map = match block {
            Value::Map(m) => m,
            _ => continue,
        };

        let block_type = match map.get("type") {
            Some(Value::String(s)) => s.as_str(),
            _ => continue,
        };

        match block_type {
            "header" => {
                let level = match map.get("level") {
                    Some(Value::Integer(n)) => *n as usize,
                    _ => 1,
                };
                let text = match map.get("text") {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                result.push(format!("{} {}", "#".repeat(level), text));
            }
            "paragraph" => {
                let text = match map.get("text") {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                result.push(text.to_string());
            }
            "list" => {
                let ordered = match map.get("ordered") {
                    Some(Value::Bool(b)) => *b,
                    _ => false,
                };
                let items = match map.get("items") {
                    Some(Value::List(items)) | Some(Value::Vector(items)) => items,
                    _ => continue,
                };

                for (i, item) in items.iter().enumerate() {
                    let text = match item {
                        Value::String(s) => s.as_str(),
                        _ => continue,
                    };
                    if ordered {
                        result.push(format!("{}. {}", i + 1, text));
                    } else {
                        result.push(format!("- {}", text));
                    }
                }
            }
            "code-block" => {
                let lang = match map.get("lang") {
                    Some(Value::String(s)) => s.as_str(),
                    Some(Value::Nil) => "",
                    _ => "",
                };
                let code = match map.get("code") {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                if lang.is_empty() {
                    result.push(format!("```\n{}\n```", code));
                } else {
                    result.push(format!("```{}\n{}\n```", lang, code));
                }
            }
            _ => {}
        }
    }

    Ok(Value::String(result.join("\n\n")))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
pub const FUNCTIONS: super::NativeFunctions = &[
    ("markdown/header", native_markdown_header),
    ("markdown/list", native_markdown_list),
    ("markdown/ordered-list", native_markdown_ordered_list),
    ("markdown/table", native_markdown_table),
    ("markdown/code-block", native_markdown_code_block),
    ("markdown/join", native_markdown_join),
    ("markdown/link", native_markdown_link),
    ("markdown/image", native_markdown_image),
    (
        "markdown/extract-code-blocks",
        native_markdown_extract_code_blocks,
    ),
    ("markdown/parse", native_markdown_parse),
    ("markdown/stringify", native_markdown_stringify),
];
