//! Markdown生成・加工関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/header", "an integer"])),
    };

    let text = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["markdown/header", "a string"])),
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/list", "a list or vector"])),
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
        _ => return Err(fmt_msg(
            MsgKey::FirstArgMustBe,
            &["markdown/ordered-list", "a list or vector"],
        )),
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/table", "a list or vector"])),
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
            _ => return Err(fmt_msg(MsgKey::MdTableRowMustBeList, &[&(i + 1).to_string()])),
        };

        if cells.len() != col_count {
            return Err(fmt_msg(
                MsgKey::MdTableColumnMismatch,
                &[&(i + 1).to_string(), &cells.len().to_string(), &col_count.to_string()],
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/code-block", "a string"])),
    };

    let code = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["markdown/code-block", "a string"])),
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/join", "a list or vector"])),
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/link", "a string"])),
    };

    let url = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["markdown/link", "a string"])),
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
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["markdown/image", "a string"])),
    };

    let src = match &args[1] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["markdown/image", "a string"])),
    };

    Ok(Value::String(format!("![{}]({})", alt, src)))
}
