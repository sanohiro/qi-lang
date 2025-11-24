use qi_lang::eval::Evaluator;
use qi_lang::i18n::{self, fmt_msg, fmt_ui_msg, ui_msg, MsgKey, UiMsg};
use qi_lang::intern;
use qi_lang::parser::Parser;
use qi_lang::project;
use qi_lang::value::{MapKey, Value};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;

#[cfg(feature = "repl")]
use colored::Colorize;

#[cfg(feature = "repl")]
use comfy_table::{presets::UTF8_FULL, Table};

#[cfg(feature = "repl")]
use notify::{RecursiveMode, Watcher};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// プロファイリング情報
#[cfg(feature = "repl")]
struct ProfileData {
    enabled: bool,
    evaluations: Vec<(std::time::Duration, usize)>, // (実行時間, 行番号)
}

#[cfg(feature = "repl")]
impl ProfileData {
    fn new() -> Self {
        ProfileData {
            enabled: false,
            evaluations: Vec::new(),
        }
    }

    fn start(&mut self) {
        self.enabled = true;
        self.evaluations.clear();
    }

    fn stop(&mut self) {
        self.enabled = false;
    }

    fn record(&mut self, duration: std::time::Duration, line_number: usize) {
        if self.enabled {
            self.evaluations.push((duration, line_number));
        }
    }

    fn report(&self) {
        if self.evaluations.is_empty() {
            println!("{}", "No profiling data available".yellow());
            return;
        }

        let total: std::time::Duration = self.evaluations.iter().map(|(d, _)| *d).sum();
        let count = self.evaluations.len();
        let avg = total / count as u32;
        let max = self.evaluations.iter().map(|(d, _)| *d).max().unwrap();
        let min = self.evaluations.iter().map(|(d, _)| *d).min().unwrap();

        println!("{}", "Profiling Report:".cyan().bold());
        println!("  Total evaluations: {}", count);
        println!("  Total time: {:?}", total);
        println!("  Average time: {:?}", avg);
        println!("  Min time: {:?}", min);
        println!("  Max time: {:?}", max);

        // 最も遅かった5つの評価を表示
        let mut sorted = self.evaluations.clone();
        sorted.sort_by(|a, b| b.0.cmp(&a.0));
        println!("\n{}", "Slowest evaluations:".yellow());
        for (i, (duration, line)) in sorted.iter().take(5).enumerate() {
            println!("  {}. Line {} - {:?}", i + 1, line, duration);
        }
    }

    fn clear(&mut self) {
        self.evaluations.clear();
    }
}

/// REPLマクロの管理
#[cfg(feature = "repl")]
struct ReplMacros {
    macros: std::collections::HashMap<String, String>,
    file_path: PathBuf,
}

#[cfg(feature = "repl")]
impl ReplMacros {
    fn new() -> Self {
        let file_path = dirs::home_dir()
            .map(|p| {
                let qi_dir = p.join(".qi");
                let _ = std::fs::create_dir_all(&qi_dir);
                qi_dir.join("macros")
            })
            .unwrap_or_else(|| std::path::PathBuf::from(".qi/macros"));

        let macros = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        ReplMacros { macros, file_path }
    }

    fn define(&mut self, name: String, command: String) {
        self.macros.insert(name, command);
        self.save();
    }

    fn delete(&mut self, name: &str) -> bool {
        let removed = self.macros.remove(name).is_some();
        if removed {
            self.save();
        }
        removed
    }

    fn get(&self, name: &str) -> Option<&String> {
        self.macros.get(name)
    }

    fn list(&self) -> Vec<(&String, &String)> {
        let mut items: Vec<_> = self.macros.iter().collect();
        items.sort_by_key(|(name, _)| *name);
        items
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.macros) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }
}

/// エラーメッセージをカラー化して表示
#[cfg(feature = "repl")]
fn print_error(error_type: &str, message: &str) {
    eprintln!("{}: {}", error_type.red().bold(), message);
}

/// エラーメッセージを通常表示（REPLなし）
#[cfg(not(feature = "repl"))]
fn print_error(error_type: &str, message: &str) {
    eprintln!("{}: {}", error_type, message);
}

/// テーブル形式で表示可能かチェックし、可能ならテーブルとして表示
#[cfg(feature = "repl")]
fn try_display_as_table(value: &Value) -> Option<String> {
    // Vectorの中身がすべてMapの場合、テーブル表示
    if let Value::Vector(vec) = value {
        if vec.is_empty() {
            return None;
        }

        // すべての要素がMapかチェック
        let all_maps = vec.iter().all(|v| matches!(v, Value::Map(_)));
        if !all_maps {
            return None;
        }

        // ヘッダーを収集（すべてのMapのキーの和集合）
        // MapKeyをそのまま保持して、検索時の型不一致を防ぐ
        let mut headers = std::collections::HashSet::new();
        for item in vec.iter() {
            if let Value::Map(m) = item {
                for key in m.keys() {
                    headers.insert(key.clone());
                }
            }
        }

        if headers.is_empty() {
            return None;
        }

        let mut header_list: Vec<MapKey> = headers.into_iter().collect();
        header_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

        // テーブルを作成
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        let header_strings: Vec<String> = header_list.iter().map(|k| k.to_string()).collect();
        table.set_header(&header_strings);

        // 各行を追加
        for item in vec.iter() {
            if let Value::Map(m) = item {
                let row: Vec<String> = header_list
                    .iter()
                    .map(|header| {
                        m.get(header)
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "".to_string())
                    })
                    .collect();
                table.add_row(row);
            }
        }

        Some(table.to_string())
    } else {
        None
    }
}

/// テーブル形式で表示（REPLなし）
#[cfg(not(feature = "repl"))]
fn try_display_as_table(_value: &Value) -> Option<String> {
    None
}

/// Pretty Print: 大きなデータ構造を見やすくフォーマット
#[cfg(feature = "repl")]
fn pretty_print_value(value: &Value, indent: usize, max_inline: usize) -> String {
    use colored::Colorize;

    match value {
        Value::Vector(vec) => {
            if vec.is_empty() {
                return "[]".to_string();
            }

            // 小さいVectorはインライン表示
            if vec.len() <= max_inline && vec.iter().all(is_simple_value) {
                let items: Vec<String> = vec.iter().map(|v| v.to_string()).collect();
                return format!("[{}]", items.join(" "));
            }

            // 大きいVectorは複数行表示
            let indent_str = " ".repeat(indent);
            let mut result = "[\n".to_string();
            for (i, item) in vec.iter().enumerate() {
                result.push_str(&format!(
                    "{}  {}",
                    indent_str,
                    pretty_print_value(item, indent + 2, max_inline)
                ));
                if i < vec.len() - 1 {
                    result.push('\n');
                }
            }
            result.push_str(&format!("\n{}]", indent_str));
            result
        }

        Value::List(lst) => {
            let vec: Vec<Value> = lst.iter().cloned().collect();
            if vec.is_empty() {
                return "()".to_string();
            }

            // 小さいListはインライン表示
            if vec.len() <= max_inline && vec.iter().all(is_simple_value) {
                let items: Vec<String> = vec.iter().map(|v| v.to_string()).collect();
                return format!("({})", items.join(" "));
            }

            // 大きいListは複数行表示
            let indent_str = " ".repeat(indent);
            let mut result = "(\n".to_string();
            for (i, item) in vec.iter().enumerate() {
                result.push_str(&format!(
                    "{}  {}",
                    indent_str,
                    pretty_print_value(item, indent + 2, max_inline)
                ));
                if i < vec.len() - 1 {
                    result.push('\n');
                }
            }
            result.push_str(&format!("\n{})", indent_str));
            result
        }

        Value::Map(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }

            // 小さいMapはインライン表示
            if map.len() <= 3 {
                let items: Vec<String> = map.iter().map(|(k, v)| format!("{} {}", k, v)).collect();
                return format!("{{{}}}", items.join(", "));
            }

            // 大きいMapは複数行表示
            let indent_str = " ".repeat(indent);
            let mut result = "{\n".to_string();
            let mut sorted_keys: Vec<_> = map.keys().collect();
            sorted_keys.sort();

            for (i, key) in sorted_keys.iter().enumerate() {
                if let Some(val) = map.get(key) {
                    result.push_str(&format!(
                        "{}  {} {}",
                        indent_str,
                        key.to_string().cyan(),
                        pretty_print_value(val, indent + 2, max_inline)
                    ));
                    if i < sorted_keys.len() - 1 {
                        result.push('\n');
                    }
                }
            }
            result.push_str(&format!("\n{}}}", indent_str));
            result
        }

        _ => value.to_string(),
    }
}

/// Pretty Print用のヘルパー: シンプルな値かどうか判定
#[cfg(feature = "repl")]
fn is_simple_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Integer(_) | Value::Float(_) | Value::Bool(_) | Value::Nil | Value::Symbol(_)
    )
}

/// Pretty Print（REPLなし）
#[cfg(not(feature = "repl"))]
fn pretty_print_value(value: &Value, _indent: usize, _max_inline: usize) -> String {
    value.to_string()
}

/// タブ補完のためのヘルパー
struct QiHelper {
    completions: HashSet<String>,
}

impl QiHelper {
    fn new() -> Self {
        let mut completions = HashSet::new();

        // REPLコマンド
        completions.insert(":help".to_string());
        completions.insert(":doc".to_string());
        completions.insert(":vars".to_string());
        completions.insert(":funcs".to_string());
        completions.insert(":builtins".to_string());
        completions.insert(":clear".to_string());
        completions.insert(":load".to_string());
        completions.insert(":reload".to_string());
        completions.insert(":watch".to_string());
        completions.insert(":unwatch".to_string());
        completions.insert(":macro".to_string());
        completions.insert(":m".to_string());
        completions.insert(":profile".to_string());
        completions.insert(":test".to_string());
        completions.insert(":trace".to_string());
        completions.insert(":untrace".to_string());
        completions.insert(":threads".to_string());
        completions.insert(":quit".to_string());

        // 特殊形式
        let special_forms = [
            "def",
            "defn",
            "defn-",
            "fn",
            "let",
            "if",
            "do",
            "loop",
            "recur",
            "match",
            "try",
            "defer",
            "quote",
            "quasiquote",
            "unquote",
            "defmacro",
            "and",
            "or",
            "not",
            "cond",
            "when",
            "unless",
        ];
        for form in &special_forms {
            completions.insert(form.to_string());
        }

        // パイプ演算子
        completions.insert("|>".to_string());
        completions.insert("|>?".to_string());
        completions.insert("||>".to_string());
        completions.insert("~>".to_string());
        completions.insert("->".to_string());
        completions.insert("=>".to_string());

        // 基本的な組み込み関数（頻出するもの）
        let common_functions = [
            // コレクション操作
            "map",
            "filter",
            "reduce",
            "first",
            "rest",
            "cons",
            "list",
            "vector",
            "count",
            "nth",
            "get",
            "assoc",
            "dissoc",
            "conj",
            "concat",
            // 文字列
            "str",
            "str/split",
            "str/join",
            "str/trim",
            "str/upper",
            "str/lower",
            // I/O
            "print",
            "println",
            "read-file",
            "write-file",
            // 数値
            "+",
            "-",
            "*",
            "/",
            "%",
            "abs",
            "min",
            "max",
            // 比較
            "=",
            "!=",
            "<",
            ">",
            "<=",
            ">=",
            // 論理
            "and",
            "or",
            "not",
            // その他
            "type",
            "nil?",
            "some?",
            "empty?",
            "range",
        ];
        for func in &common_functions {
            completions.insert(func.to_string());
        }

        QiHelper { completions }
    }

    fn update_completions(&mut self, evaluator: &Evaluator) {
        // 環境から変数と関数を取得
        if let Some(env) = evaluator.get_env() {
            let env = env.read();
            for (name, _) in env.bindings() {
                self.completions.insert(name.to_string());
            }
        }
    }
}

impl Completer for QiHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace() || c == '(' || c == '[' || c == '{')
            .map(|i| i + 1)
            .unwrap_or(0);

        let word = &line[start..pos];

        let mut matches: Vec<Pair> = self
            .completions
            .iter()
            .filter(|s| s.starts_with(word))
            .map(|s| Pair {
                display: s.clone(),
                replacement: s.clone(),
            })
            .collect();

        matches.sort_by(|a, b| a.display.cmp(&b.display));

        Ok((start, matches))
    }
}

impl Highlighter for QiHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        use colored::Colorize;

        // 特殊形式（キーワード）
        let special_forms = [
            "def",
            "defn",
            "defn-",
            "fn",
            "let",
            "if",
            "do",
            "loop",
            "recur",
            "match",
            "try",
            "defer",
            "quote",
            "quasiquote",
            "unquote",
            "defmacro",
            "and",
            "or",
            "not",
            "cond",
            "when",
            "unless",
        ];

        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut current_word = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let mut escape_next = false;
        let mut paren_depth = 0; // 括弧のネストレベル

        while let Some(ch) = chars.next() {
            // コメント処理
            if !in_string && ch == ';' && chars.peek() == Some(&';') {
                in_comment = true;
                result.push_str(&format!("{}", ch.to_string().bright_black()));
                continue;
            }

            if in_comment {
                result.push_str(&format!("{}", ch.to_string().bright_black()));
                continue;
            }

            // 文字列処理
            if ch == '"' && !escape_next {
                in_string = !in_string;
                result.push_str(&format!("{}", ch.to_string().yellow()));
                escape_next = false;
                continue;
            }

            if in_string {
                escape_next = ch == '\\' && !escape_next;
                result.push_str(&format!("{}", ch.to_string().yellow()));
                continue;
            }

            // 単語の区切り文字
            if ch.is_whitespace()
                || ch == '('
                || ch == ')'
                || ch == '['
                || ch == ']'
                || ch == '{'
                || ch == '}'
            {
                // 蓄積された単語を処理
                if !current_word.is_empty() {
                    result.push_str(&highlight_word(&current_word, &special_forms));
                    current_word.clear();
                }

                // 括弧の色付け（ネストレベルに応じて色を変える）
                if ch == '(' || ch == '[' || ch == '{' {
                    result.push_str(&colorize_paren(ch, paren_depth));
                    paren_depth += 1;
                } else if ch == ')' || ch == ']' || ch == '}' {
                    paren_depth = paren_depth.saturating_sub(1);
                    result.push_str(&colorize_paren(ch, paren_depth));
                } else {
                    result.push(ch);
                }
            } else {
                current_word.push(ch);
            }
        }

        // 最後の単語を処理
        if !current_word.is_empty() {
            result.push_str(&highlight_word(&current_word, &special_forms));
        }

        std::borrow::Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        // ヒントを薄いグレーで表示（ANSI: 90 = bright black）
        std::borrow::Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

/// 単語をハイライト
fn highlight_word(word: &str, special_forms: &[&str]) -> String {
    use colored::Colorize;

    // キーワード（:で始まる）
    if word.starts_with(':') {
        return word.cyan().to_string();
    }

    // 数値
    if word.parse::<i64>().is_ok() || word.parse::<f64>().is_ok() {
        return word.bright_cyan().to_string();
    }

    // パイプ演算子
    if matches!(word, "|>" | "|>?" | "||>" | "~>" | "->" | "=>") {
        return word.bright_magenta().bold().to_string();
    }

    // 特殊形式
    if special_forms.contains(&word) {
        return word.blue().bold().to_string();
    }

    // その他（通常表示）
    word.to_string()
}

/// 括弧を色付け（ネストレベルに応じて色を変える）
fn colorize_paren(ch: char, depth: usize) -> String {
    use colored::Colorize;

    // 6色のレインボーカラー（レベルごとにローテーション）
    let colors = [
        |s: &str| s.bright_red().to_string(),
        |s: &str| s.bright_green().to_string(),
        |s: &str| s.bright_yellow().to_string(),
        |s: &str| s.bright_blue().to_string(),
        |s: &str| s.bright_magenta().to_string(),
        |s: &str| s.bright_cyan().to_string(),
    ];

    let color_fn = colors[depth % colors.len()];
    color_fn(&ch.to_string())
}

impl Hinter for QiHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        // 履歴から候補を検索
        if line.is_empty() || pos < line.len() {
            return None;
        }

        // 履歴を逆順で検索して、現在の入力で始まるものを見つける
        let history = ctx.history();
        for i in (0..history.len()).rev() {
            if let Ok(Some(entry)) = history.get(i, rustyline::history::SearchDirection::Forward) {
                let entry_str = entry.entry;
                if entry_str.starts_with(line) && entry_str.len() > line.len() {
                    // 残りの部分だけを返す
                    return Some(entry_str[line.len()..].to_string());
                }
            }
        }
        None
    }
}

impl Validator for QiHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let input = ctx.input();

        // 空行は常に有効
        if input.trim().is_empty() {
            return Ok(rustyline::validate::ValidationResult::Valid(None));
        }

        // REPLコマンド（:で始まる）は常に1行で完結
        if input.trim_start().starts_with(':') {
            return Ok(rustyline::validate::ValidationResult::Valid(None));
        }

        // 括弧のバランスをチェック
        if is_balanced(input) {
            Ok(rustyline::validate::ValidationResult::Valid(None))
        } else {
            // 不完全な入力は次の行を待つ
            Ok(rustyline::validate::ValidationResult::Incomplete)
        }
    }
}

impl Helper for QiHelper {}

fn main() {
    // 国際化システムを初期化
    i18n::init();

    let args: Vec<String> = std::env::args().collect();

    // コマンドライン引数の解析
    if args.len() == 1 {
        // 引数なし: REPLを起動
        repl(None, false);
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        }
        "-v" | "--version" => {
            println!("{}", fmt_ui_msg(UiMsg::VersionString, &[VERSION]));
        }
        "--upgrade" => match qi_lang::upgrade::upgrade() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        #[cfg(feature = "dap-server")]
        "--dap" => {
            // DAPサーバーを起動（非同期版、stdin/stdoutで通信）
            if let Err(e) = qi_lang::dap::DapServer::run_async() {
                eprintln!("{}", fmt_msg(MsgKey::DapServerError, &[&e.to_string()]));
                std::process::exit(1);
            }
        }
        #[cfg(not(feature = "dap-server"))]
        "--dap" => {
            eprintln!("{}", fmt_msg(MsgKey::DapServerNotEnabled, &[]));
            std::process::exit(1);
        }
        "-q" | "--quiet" => {
            // quietモードでREPL起動
            repl(None, true);
        }
        "new" => {
            // プロジェクト作成
            if args.len() < 3 {
                eprintln!("{}", fmt_ui_msg(UiMsg::ProjectNewNeedName, &[]));
                eprintln!("{}", fmt_ui_msg(UiMsg::ProjectNewUsage, &[]));
                std::process::exit(1);
            }
            let project_name = args[2].clone();

            // テンプレートオプションのパース
            let mut template = None;
            let mut i = 3;
            while i < args.len() {
                if (args[i] == "--template" || args[i] == "-t") && i + 1 < args.len() {
                    template = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!(
                        "{}",
                        fmt_ui_msg(UiMsg::ProjectNewUnknownOption, &[&args[i]])
                    );
                    std::process::exit(1);
                }
            }

            if let Err(e) = project::new_project(project_name, template) {
                eprintln!("{}", fmt_ui_msg(UiMsg::ProjectNewError, &[&e]));
                std::process::exit(1);
            }
        }
        "template" => {
            // テンプレート管理
            if args.len() < 3 {
                eprintln!("{}", fmt_ui_msg(UiMsg::TemplateNeedSubcommand, &[]));
                eprintln!("{}", fmt_ui_msg(UiMsg::TemplateUsage, &[]));
                std::process::exit(1);
            }

            match args[2].as_str() {
                "list" => {
                    if let Err(e) = project::list_templates() {
                        eprintln!("{}", fmt_ui_msg(UiMsg::ProjectNewError, &[&e]));
                        std::process::exit(1);
                    }
                }
                "info" => {
                    if args.len() < 4 {
                        eprintln!("{}", fmt_ui_msg(UiMsg::TemplateNeedName, &[]));
                        eprintln!("{}", fmt_ui_msg(UiMsg::TemplateInfoUsage, &[]));
                        std::process::exit(1);
                    }
                    if let Err(e) = project::show_template_info(&args[3]) {
                        eprintln!("{}", fmt_ui_msg(UiMsg::ProjectNewError, &[&e]));
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!(
                        "{}",
                        fmt_ui_msg(UiMsg::TemplateUnknownSubcommand, &[&args[2]])
                    );
                    eprintln!("{}", fmt_ui_msg(UiMsg::TemplateUsage, &[]));
                    std::process::exit(1);
                }
            }
        }
        "test" => {
            // テスト実行
            run_tests(&args[2..]);
        }
        "-e" | "-c" => {
            // ワンライナー実行
            if args.len() < 3 {
                eprintln!("{}", fmt_ui_msg(UiMsg::ErrorRequiresArg, &[&args[1]]));
                std::process::exit(1);
            }
            run_code(&args[2]);
        }
        "-" => {
            // 標準入力からスクリプト実行
            run_stdin();
        }
        "-l" | "--load" => {
            // REPLでファイルをロード
            if args.len() < 3 {
                eprintln!("{}", fmt_ui_msg(UiMsg::ErrorRequiresFile, &[&args[1]]));
                std::process::exit(1);
            }
            // -l はREPLなので quiet=false
            repl(Some(&args[2]), false);
        }
        arg if arg.starts_with('-') => {
            eprintln!("{}", fmt_ui_msg(UiMsg::ErrorUnknownOption, &[arg]));
            eprintln!("{}", ui_msg(UiMsg::ErrorUseHelp));
            std::process::exit(1);
        }
        _ => {
            // ファイルを実行
            run_file(&args[1]);
        }
    }
}

fn print_help() {
    println!("{}", ui_msg(UiMsg::HelpTitle));
    println!();
    println!("{}", ui_msg(UiMsg::HelpUsage));
    println!();
    println!("{}:", ui_msg(UiMsg::HelpOptions));
    println!("{}", ui_msg(UiMsg::OptNew));
    println!("{}", ui_msg(UiMsg::OptTemplate));
    println!("{}", ui_msg(UiMsg::OptExecute));
    println!("{}", ui_msg(UiMsg::OptStdin));
    println!("{}", ui_msg(UiMsg::OptLoad));
    println!("{}", ui_msg(UiMsg::OptQuiet));
    #[cfg(feature = "dap-server")]
    println!("{}", ui_msg(UiMsg::OptDap));
    println!("{}", ui_msg(UiMsg::OptHelp));
    println!("{}", ui_msg(UiMsg::OptVersion));
    println!();
    println!("{}:", ui_msg(UiMsg::HelpExamples));
    println!("{}", ui_msg(UiMsg::ExampleStartRepl));
    println!("{}", ui_msg(UiMsg::ExampleNewProject));
    println!("{}", ui_msg(UiMsg::ExampleNewHttpServer));
    println!("{}", ui_msg(UiMsg::ExampleTemplateList));
    println!("{}", ui_msg(UiMsg::ExampleRunScript));
    println!("{}", ui_msg(UiMsg::ExampleExecuteCode));
    println!("{}", ui_msg(UiMsg::ExampleStdin));
    println!("{}", ui_msg(UiMsg::ExampleLoadFile));
    println!();
    println!("{}:", ui_msg(UiMsg::HelpEnvVars));
    println!("{}", ui_msg(UiMsg::EnvLangQi));
    println!("{}", ui_msg(UiMsg::EnvLangSystem));
}

fn run_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
        std::process::exit(1);
    });

    let mut evaluator = Evaluator::new();
    eval_code(&mut evaluator, &content, false, Some(path));
}

/// テストを実行
fn run_tests(args: &[String]) {
    use std::path::Path;
    use std::time::Instant;

    let mut test_files = Vec::new();

    // 引数がある場合は指定されたファイルを実行
    if !args.is_empty() {
        for arg in args {
            test_files.push(arg.clone());
        }
    } else {
        // 引数がない場合は tests/ ディレクトリ内の *.qi ファイルを検索
        let test_dir = Path::new("tests");
        if test_dir.exists() && test_dir.is_dir() {
            match std::fs::read_dir(test_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("qi") {
                            if let Some(path_str) = path.to_str() {
                                test_files.push(path_str.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading tests directory: {}", e);
                    std::process::exit(1);
                }
            }
        }

        test_files.sort();
    }

    if test_files.is_empty() {
        println!("no test files found");
        return;
    }

    println!("running {} test files\n", test_files.len());

    let evaluator = Evaluator::new();
    let start_time = Instant::now();

    // テストファイルを順次実行
    for test_file in &test_files {
        let content = match std::fs::read_to_string(test_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", test_file, e);
                continue;
            }
        };

        // テストファイルを評価（エラーが出ても続行）
        evaluator.set_source(test_file.clone(), content.clone());
        match Parser::new(&content) {
            Ok(mut parser) => {
                parser.set_source_name(test_file.clone());
                match parser.parse_all() {
                    Ok(exprs) => {
                        for expr in exprs.iter() {
                            if let Err(e) = evaluator.eval(expr) {
                                eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e);
            }
        }
    }

    // test/run-all を呼び出して結果を表示
    match Parser::new("(test/run-all)") {
        Ok(mut parser) => match parser.parse() {
            Ok(expr) => {
                let elapsed = start_time.elapsed();
                match evaluator.eval(&expr) {
                    Ok(_) => {
                        println!("\nfinished in {:.2}s", elapsed.as_secs_f64());
                        std::process::exit(0);
                    }
                    Err(_) => {
                        println!("\nfinished in {:.2}s", elapsed.as_secs_f64());
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", fmt_msg(MsgKey::InternalError, &[&e]));
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", fmt_msg(MsgKey::InternalError, &[&e]));
            std::process::exit(1);
        }
    }
}

/// .qi/init.qi を読み込む（あれば）
fn load_init_file(evaluator: &Evaluator) {
    // ~/.qi/init.qi の優先度が最高
    if let Some(home_dir) = dirs::home_dir() {
        let user_init = home_dir.join(".qi/init.qi");
        if user_init.exists() {
            if let Ok(content) = std::fs::read_to_string(&user_init) {
                eval_repl_code(evaluator, &content, Some(&user_init.display().to_string()));
            }
        }
    }

    // ./.qi/init.qi（プロジェクトローカル）
    let local_init = PathBuf::from(".qi/init.qi");
    if local_init.exists() {
        if let Ok(content) = std::fs::read_to_string(&local_init) {
            eval_repl_code(evaluator, &content, Some(&local_init.display().to_string()));
        }
    }
}

/// セッション変数を~/.qi/session.qiから復元
#[cfg(feature = "repl")]
fn load_session(evaluator: &Evaluator) {
    if let Some(home_dir) = dirs::home_dir() {
        let session_file = home_dir.join(".qi/session.qi");
        if session_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&session_file) {
                eval_repl_code(evaluator, &content, Some("<session>"));
            }
        }
    }
}

fn run_code(code: &str) {
    let mut evaluator = Evaluator::new();

    // .qi/init.qi を読み込む
    load_init_file(&evaluator);

    // パイプから入力がある場合、stdinを自動バインド
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        // 標準入力から全行読み込み
        match qi_lang::builtins::io::native_stdin_read_lines(&[]) {
            Ok(lines) => {
                // グローバル変数'stdin'として環境にバインド
                if let Some(env) = evaluator.get_env() {
                    env.write().set("stdin", lines);
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    fmt_msg(MsgKey::IoReadLinesFailedToRead, &["stdin", &e])
                );
            }
        }
    }

    eval_code(&mut evaluator, code, true, None);
}

fn run_stdin() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToReadStdin), e);
        std::process::exit(1);
    }

    let mut evaluator = Evaluator::new();
    eval_code(&mut evaluator, &input, false, Some("<stdin>"));
}

fn eval_code(evaluator: &mut Evaluator, code: &str, print_result: bool, filename: Option<&str>) {
    // ソース情報を設定
    let source_name = filename.unwrap_or("<code>").to_string();
    evaluator.set_source(source_name.clone(), code.to_string());

    match Parser::new(code) {
        Ok(mut parser) => {
            parser.set_source_name(source_name);
            match parser.parse_all() {
                Ok(exprs) => {
                    for (i, expr) in exprs.iter().enumerate() {
                        match evaluator.eval(expr) {
                            Ok(value) => {
                                // ワンライナーの場合、最後の結果を表示
                                if print_result && i == exprs.len() - 1 {
                                    #[cfg(feature = "repl")]
                                    {
                                        println!("{}", pretty_print_value(&value, 0, 10));
                                    }
                                    #[cfg(not(feature = "repl"))]
                                    println!("{}", value);
                                }
                            }
                            Err(e) => {
                                print_error(ui_msg(UiMsg::ErrorRuntime), &e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    print_error(ui_msg(UiMsg::ErrorParse), &e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            print_error(ui_msg(UiMsg::ErrorLexer), &e);
            std::process::exit(1);
        }
    }
}

fn repl(preload: Option<&str>, quiet: bool) {
    if !quiet {
        println!("{}", fmt_ui_msg(UiMsg::ReplWelcome, &[VERSION]));
        println!("{}", ui_msg(UiMsg::ReplPressCtrlC));
        println!("{}", ui_msg(UiMsg::ReplTypeHelp));
        println!();
    }

    let mut evaluator = Evaluator::new();

    // .qi/init.qi を読み込む
    load_init_file(&evaluator);

    // セッション変数を復元
    #[cfg(feature = "repl")]
    load_session(&evaluator);

    let mut helper = QiHelper::new();

    // REPL設定
    let config = rustyline::Config::builder()
        .bracketed_paste(true) // 括弧付きペーストモード（大量コピペ対応）
        .edit_mode(rustyline::EditMode::Emacs) // Emacsモード（デフォルト）
        .auto_add_history(true) // 自動履歴追加
        .history_ignore_dups(true) // 重複履歴を無視
        .expect("Failed to set history_ignore_dups")
        .build();

    let mut rl = Editor::with_config(config).expect("Failed to create editor");
    rl.set_helper(Some(helper));

    // キーバインド追加
    rl.bind_sequence(
        rustyline::KeyEvent::alt('n'),
        rustyline::Cmd::HistorySearchForward,
    );
    rl.bind_sequence(
        rustyline::KeyEvent::alt('p'),
        rustyline::Cmd::HistorySearchBackward,
    );

    // Ctrl+R: 履歴検索（デフォルトで有効）
    // Ctrl+_: Undo（デフォルトで有効）
    // Alt+f: 次の単語へ移動（デフォルトで有効）
    // Alt+b: 前の単語へ移動（デフォルトで有効）

    // .qi/history ファイルのパス
    let history_file = dirs::home_dir()
        .map(|p| {
            let qi_dir = p.join(".qi");
            // .qiディレクトリがなければ作成
            let _ = std::fs::create_dir_all(&qi_dir);
            qi_dir.join("history")
        })
        .unwrap_or_else(|| std::path::PathBuf::from(".qi/history"));

    let _ = rl.load_history(&history_file);

    let mut last_loaded_file: Option<String> = None;

    // ファイル監視用のチャンネル（ホットリロード）
    #[cfg(feature = "repl")]
    let (watch_tx, watch_rx) = std::sync::mpsc::channel::<String>();
    #[cfg(feature = "repl")]
    let watched_files = std::sync::Arc::new(parking_lot::Mutex::new(std::collections::HashSet::<
        String,
    >::new()));
    // スレッド管理: パス -> (JoinHandle, 終了シグナル送信側)
    #[cfg(feature = "repl")]
    let watch_threads = std::sync::Arc::new(parking_lot::Mutex::new(
        std::collections::HashMap::<String, std::sync::mpsc::Sender<()>>::new(),
    ));

    // REPLマクロ
    #[cfg(feature = "repl")]
    let mut macros = ReplMacros::new();

    // プロファイリングデータ
    #[cfg(feature = "repl")]
    let mut profile = ProfileData::new();

    // ファイルのプリロード
    if let Some(path) = preload {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                println!("{}", fmt_ui_msg(UiMsg::ReplLoading, &[path]));
                eval_code(&mut evaluator, &content, false, Some(path));
                println!("{}\n", ui_msg(UiMsg::ReplLoaded));
                last_loaded_file = Some(path.to_string());
            }
            Err(e) => {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
                std::process::exit(1);
            }
        }
    }

    // ビルトイン関数を補完候補に追加
    if let Some(h) = rl.helper_mut() {
        h.update_completions(&evaluator);
    }

    let mut line_number = 1;
    let mut result_number = 1;

    loop {
        // ファイル変更通知をチェック（ホットリロード）
        #[cfg(feature = "repl")]
        if let Ok(changed_file) = watch_rx.try_recv() {
            println!(
                "{}",
                format!("\n[File changed: {}]", changed_file).bright_black()
            );
            match std::fs::read_to_string(&changed_file) {
                Ok(content) => {
                    eval_repl_code(&evaluator, &content, Some(&changed_file));
                    println!("{}", "[Reloaded]".green());
                }
                Err(e) => {
                    eprintln!("{}: {}", "Failed to reload".red(), e);
                }
            }
        }

        let prompt = format!("qi:{}> ", line_number);

        let readline = rl.readline(&prompt);

        match readline {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                // REPLコマンドの処理
                if input.starts_with(':') {
                    rl.add_history_entry(input).ok();
                    handle_repl_command(
                        input,
                        &evaluator,
                        &mut last_loaded_file,
                        #[cfg(feature = "repl")]
                        &mut macros,
                        #[cfg(feature = "repl")]
                        &watch_tx,
                        #[cfg(feature = "repl")]
                        &watched_files,
                        #[cfg(feature = "repl")]
                        &watch_threads,
                        #[cfg(feature = "repl")]
                        &mut profile,
                    );

                    // :clear の場合は evaluator と helper をリセット
                    if input == ":clear" {
                        evaluator = Evaluator::new();
                        helper = QiHelper::new();
                        rl.set_helper(Some(helper));
                    } else if input == ":quit" {
                        break;
                    } else {
                        // 補完候補を更新
                        if let Some(h) = rl.helper_mut() {
                            h.update_completions(&evaluator);
                        }
                    }
                    continue;
                }

                // 通常の式を評価
                rl.add_history_entry(input).ok();

                match Parser::new(input) {
                    Ok(mut parser) => match parser.parse() {
                        Ok(expr) => {
                            // 評価時間を測定
                            let start = std::time::Instant::now();
                            let eval_result = evaluator.eval(&expr);
                            let elapsed = start.elapsed();

                            match eval_result {
                                Ok(value) => {
                                    // 結果ラベル
                                    let result_label = format!("${}", result_number);

                                    // テーブル表示を試みる
                                    if let Some(table) = try_display_as_table(&value) {
                                        println!("{} =>", result_label.green().bold());
                                        println!("{}", table);
                                    } else {
                                        // Pretty Print（大きなデータは複数行表示）
                                        #[cfg(feature = "repl")]
                                        {
                                            let formatted = pretty_print_value(&value, 0, 10);
                                            if formatted.contains('\n') {
                                                println!("{} =>", result_label.green().bold());
                                                println!("{}", formatted);
                                            } else {
                                                println!(
                                                    "{} => {}",
                                                    result_label.green().bold(),
                                                    formatted
                                                );
                                            }
                                        }
                                        #[cfg(not(feature = "repl"))]
                                        println!("{} => {}", result_label.green().bold(), value);
                                    }

                                    // 評価時間を表示（100ms以上の場合のみ）
                                    if elapsed.as_millis() >= 100 {
                                        println!(
                                            "{}",
                                            format!("({}ms)", elapsed.as_millis()).bright_black()
                                        );
                                    }

                                    // 結果を環境に登録
                                    if let Some(env) = evaluator.get_env() {
                                        env.write().set(result_label.as_str(), value.clone());
                                    }

                                    // プロファイリングに記録
                                    #[cfg(feature = "repl")]
                                    profile.record(elapsed, line_number);

                                    line_number += 1;
                                    result_number += 1;

                                    // 補完候補を更新
                                    if let Some(h) = rl.helper_mut() {
                                        h.update_completions(&evaluator);
                                    }
                                }
                                Err(e) => print_error(ui_msg(UiMsg::ErrorRuntime), &e),
                            }
                        }
                        Err(e) => print_error(ui_msg(UiMsg::ErrorParse), &e),
                    },
                    Err(e) => print_error(ui_msg(UiMsg::ErrorLexer), &e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("{}: {:?}", ui_msg(UiMsg::ErrorInput), err);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_file);

    // セッション変数を保存
    save_session(&evaluator);

    if !quiet {
        println!("\n{}", ui_msg(UiMsg::ReplGoodbye));
    }
}

/// セッション変数を~/.qi/session.qiに保存
#[cfg(feature = "repl")]
fn save_session(evaluator: &Evaluator) {
    use std::io::Write;

    if let Some(env) = evaluator.get_env() {
        let env = env.read();

        // ユーザー定義変数のみを抽出（組み込み関数や__doc__を除外）
        let user_vars: Vec<(String, Value)> = env
            .bindings()
            .filter(|(name, val)| {
                // 関数は除外（復元不可能）
                if matches!(
                    val,
                    Value::NativeFunc(_) | Value::Function { .. } | Value::Macro { .. }
                ) {
                    return false;
                }

                // 除外パターン
                !name.starts_with("__")  // ドキュメント等
                && !name.starts_with("$")  // REPL結果変数
                && !name.contains('/')  // 名前空間付き関数（str/, list/等）
                && !matches!(name.as_ref(),
                    "+" | "-" | "*" | "/" | "%" | "=" | "!=" | "<" | ">" | "<=" | ">=" |
                    "and" | "or" | "not" | "nil" | "true" | "false" |
                    "def" | "defn" | "fn" | "let" | "if" | "do" | "loop" | "recur" |
                    "match" | "try" | "defer" | "use" | "export" | "quote" |
                    "list" | "vector" | "map" | "set" | "first" | "rest" | "cons" |
                    "count" | "nth" | "get" | "assoc" | "dissoc" | "keys" | "vals" |
                    "filter" | "reduce" | "range" | "reverse" | "sort" |
                    "print" | "println" | "read-line" | "slurp" | "spit" |
                    "type" | "eval" | "apply" | "partial" | "comp" | "identity" |
                    "macroexpand" | "gensym" | "source" | "stdin"
                )
            })
            .map(|(name, val)| (name.to_string(), val.clone()))
            .collect();

        if user_vars.is_empty() {
            return; // 保存する変数がない
        }

        // ~/.qi/session.qiに保存
        if let Some(home) = dirs::home_dir() {
            let qi_dir = home.join(".qi");
            let session_file = qi_dir.join("session.qi");

            // .qiディレクトリが存在しない場合は作成
            let _ = std::fs::create_dir_all(&qi_dir);

            if let Ok(mut file) = std::fs::File::create(&session_file) {
                let _ = writeln!(file, ";; Qi REPL Session (自動生成 - 手動編集可)");
                let _ = writeln!(file, ";; このファイルはREPL終了時に自動保存されます\n");

                for (name, val) in user_vars {
                    let _ = writeln!(file, "(def {} {})", name, val);
                }
            }
        }
    }
}

/// 括弧のバランスをチェック
fn is_balanced(input: &str) -> bool {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            escape = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '(' | '[' | '{' if !in_string => depth += 1,
            ')' | ']' | '}' if !in_string => depth -= 1,
            _ => {}
        }
    }

    depth == 0 && !in_string
}

/// ドキュメントパス情報
struct DocPaths {
    base_path: PathBuf,
    files: Vec<PathBuf>,
    lang: String,
}

/// ドキュメントパスの探索（LazyLockで自動的に1回だけ実行される）
static DOC_PATHS: LazyLock<Option<DocPaths>> = LazyLock::new(|| {
    let lang = std::env::var("QI_LANG").unwrap_or_else(|_| "en".to_string());

    // ドキュメントディレクトリの候補パスを取得
    let mut doc_base_paths = vec![PathBuf::from("std/docs")];
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            doc_base_paths.push(exe_dir.join("std/docs"));
        }
    }

    // 最初に見つかったディレクトリからドキュメントファイル一覧を取得
    let mut doc_files = HashSet::new();
    let mut found_base = None;

    for base_path in &doc_base_paths {
        // 英語版ディレクトリから取得
        let en_path = base_path.join("en");
        if let Ok(entries) = std::fs::read_dir(&en_path) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                if let Some(ext) = filename.to_str() {
                    if ext.ends_with(".qi") {
                        doc_files.insert(filename.into());
                    }
                }
            }
            found_base = Some(base_path.clone());
        }

        // 指定言語版ディレクトリからも取得（enと異なる場合）
        if lang != "en" && found_base.is_some() {
            let lang_path = base_path.join(&lang);
            if let Ok(entries) = std::fs::read_dir(&lang_path) {
                for entry in entries.flatten() {
                    let filename = entry.file_name();
                    if let Some(ext) = filename.to_str() {
                        if ext.ends_with(".qi") {
                            doc_files.insert(filename.into());
                        }
                    }
                }
            }
        }

        if found_base.is_some() {
            break;
        }
    }

    // ドキュメントが見つかった場合、パス情報を返す
    found_base.map(|base_path| {
        let mut files: Vec<_> = doc_files.into_iter().collect();
        files.sort();
        DocPaths {
            base_path,
            files,
            lang,
        }
    })
});

/// 標準ライブラリドキュメントの遅延ロード（スレッドセーフ）
///
/// LazyLockでパス探索を1回だけ実行し、評価は各Evaluatorごとに行う
/// これにより、複数のEvaluatorが生成されてもそれぞれがドキュメントをロード可能
fn lazy_load_std_docs(evaluator: &Evaluator) {
    if let Some(paths) = &*DOC_PATHS {
        load_docs_from_paths(evaluator, paths);
    }
}

/// パス情報からドキュメントをロード
fn load_docs_from_paths(evaluator: &Evaluator, paths: &DocPaths) {
    // 1. 英語版を先に読み込み
    for file in &paths.files {
        let en_doc_path = paths.base_path.join("en").join(file);
        if let Ok(content) = std::fs::read_to_string(&en_doc_path) {
            let path_str = en_doc_path.display().to_string();
            eval_repl_code(evaluator, &content, Some(&path_str));
        }
    }

    // 2. 指定言語版を読み込み（enと同じ場合はスキップ）
    if paths.lang != "en" {
        for file in &paths.files {
            let lang_doc_path = paths.base_path.join(&paths.lang).join(file);
            if let Ok(content) = std::fs::read_to_string(&lang_doc_path) {
                let path_str = lang_doc_path.display().to_string();
                eval_repl_code(evaluator, &content, Some(&path_str));
            }
        }
    }
}

/// REPLコマンドの処理
fn handle_repl_command(
    cmd: &str,
    evaluator: &Evaluator,
    last_loaded_file: &mut Option<String>,
    #[cfg(feature = "repl")] macros: &mut ReplMacros,
    #[cfg(feature = "repl")] watch_tx: &std::sync::mpsc::Sender<String>,
    #[cfg(feature = "repl")] watched_files: &std::sync::Arc<
        parking_lot::Mutex<std::collections::HashSet<String>>,
    >,
    #[cfg(feature = "repl")] watch_threads: &std::sync::Arc<
        parking_lot::Mutex<std::collections::HashMap<String, std::sync::mpsc::Sender<()>>>,
    >,
    #[cfg(feature = "repl")] profile: &mut ProfileData,
) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0];

    match command {
        ":help" => {
            println!("{}", ui_msg(UiMsg::ReplAvailableCommands));
            println!("  {}", ui_msg(UiMsg::ReplCommandHelp));
            println!("  {}", ui_msg(UiMsg::ReplCommandDoc));
            println!("  {}", ui_msg(UiMsg::ReplCommandVars));
            println!("  {}", ui_msg(UiMsg::ReplCommandFuncs));
            println!("  {}", ui_msg(UiMsg::ReplCommandBuiltins));
            println!("  {}", ui_msg(UiMsg::ReplCommandClear));
            println!("  {}", ui_msg(UiMsg::ReplCommandLoad));
            println!("  {}", ui_msg(UiMsg::ReplCommandReload));
            println!("  {}", ui_msg(UiMsg::ReplCommandQuit));
        }
        ":vars" => {
            if let Some(env) = evaluator.get_env() {
                let env = env.read();
                let mut vars: Vec<_> = env
                    .bindings()
                    .filter(|(_, v)| !matches!(v, Value::Function(_) | Value::NativeFunc(_)))
                    .map(|(name, _)| name.clone())
                    .collect();
                vars.sort();

                if vars.is_empty() {
                    println!("{}", ui_msg(UiMsg::ReplNoVariables));
                } else {
                    println!("{}", ui_msg(UiMsg::ReplDefinedVariables));
                    for var in vars {
                        println!("  {}", var);
                    }
                }
            }
        }
        ":funcs" => {
            if let Some(env) = evaluator.get_env() {
                let env = env.read();
                let mut funcs: Vec<_> = env
                    .bindings()
                    .filter(|(_, v)| matches!(v, Value::Function(_)))
                    .map(|(name, _)| name.clone())
                    .collect();
                funcs.sort();

                if funcs.is_empty() {
                    println!("{}", ui_msg(UiMsg::ReplNoFunctions));
                } else {
                    println!("{}", ui_msg(UiMsg::ReplDefinedFunctions));
                    for func in funcs {
                        println!("  {}", func);
                    }
                }
            }
        }
        ":doc" => {
            // 初回実行時に標準ライブラリドキュメントをロード
            lazy_load_std_docs(evaluator);

            if parts.len() < 2 {
                eprintln!("{}", fmt_ui_msg(UiMsg::ReplDocUsage, &[]));
                return;
            }

            let name = parts[1];
            if let Some(env) = evaluator.get_env() {
                let env = env.read();

                // 関数/変数の存在確認
                if env.get(name).is_none() {
                    eprintln!("{}", fmt_ui_msg(UiMsg::ReplDocNotFound, &[name]));
                    return;
                }

                // ドキュメント取得（多言語対応）
                // 優先順位:
                // - LANG=ja: __doc__name_ja → __doc__name → __doc__name_en
                // - LANG=en: __doc__name_en → __doc__name
                // - LANG=fr: __doc__name_fr → __doc__name → __doc__name_en
                let lang = std::env::var("QI_LANG").unwrap_or_else(|_| "en".to_string());
                let doc_key_lang = format!("{}{}_{}", qi_lang::eval::DOC_PREFIX, name, lang);
                let doc_key = format!("{}{}", qi_lang::eval::DOC_PREFIX, name);

                let doc = env
                    .get(&doc_key_lang)
                    .or_else(|| env.get(&doc_key))
                    .or_else(|| {
                        // enの場合はenフォールバックをスキップ（既にチェック済み）
                        if lang != "en" {
                            env.get(&format!("{}{}_en", qi_lang::eval::DOC_PREFIX, name))
                        } else {
                            None
                        }
                    });

                match doc {
                    Some(Value::String(s)) => {
                        println!("\n{}: {}\n", name.green().bold(), s);
                    }
                    Some(Value::Map(m)) => {
                        // 構造化ドキュメント
                        println!("\n{}:", name.green().bold());

                        // Description
                        if let Some(Value::String(desc)) =
                            m.get(&MapKey::Keyword(intern::intern_keyword("desc")))
                        {
                            println!("  {}", desc);
                        }

                        // Parameters
                        if let Some(Value::Vector(params)) =
                            m.get(&MapKey::Keyword(intern::intern_keyword("params")))
                        {
                            println!("\n{}", ui_msg(UiMsg::ReplDocParameters).cyan());
                            for param in params {
                                if let Value::Map(pm) = param {
                                    let pname =
                                        pm.get(&MapKey::Keyword(intern::intern_keyword("name")));
                                    let ptype =
                                        pm.get(&MapKey::Keyword(intern::intern_keyword("type")));
                                    let pdesc =
                                        pm.get(&MapKey::Keyword(intern::intern_keyword("desc")));

                                    if let Some(Value::String(name_str)) = pname {
                                        print!("  {}", name_str.yellow());
                                        if let Some(Value::String(type_str)) = ptype {
                                            print!(" ({})", type_str.bright_black());
                                        }
                                        if let Some(Value::String(desc_str)) = pdesc {
                                            print!(" - {}", desc_str);
                                        }
                                        println!();
                                    }
                                }
                            }
                        }

                        // Returns
                        if let Some(returns) =
                            m.get(&MapKey::Keyword(intern::intern_keyword("returns")))
                        {
                            println!("\n{}", "Returns:".cyan());
                            match returns {
                                Value::String(s) => println!("  {}", s),
                                Value::Map(rm) => {
                                    if let Some(Value::String(rtype)) =
                                        rm.get(&MapKey::Keyword(intern::intern_keyword("type")))
                                    {
                                        print!("  {}", rtype.yellow());
                                    }
                                    if let Some(Value::String(rdesc)) =
                                        rm.get(&MapKey::Keyword(intern::intern_keyword("desc")))
                                    {
                                        print!(" - {}", rdesc);
                                    }
                                    println!();
                                }
                                _ => {}
                            }
                        }

                        // Examples
                        if let Some(Value::Vector(examples)) =
                            m.get(&MapKey::Keyword(intern::intern_keyword("examples")))
                        {
                            println!("\n{}", ui_msg(UiMsg::ReplDocExamples).cyan());
                            for ex in examples {
                                if let Value::String(s) = ex {
                                    println!("  {}", s.bright_black());
                                }
                            }
                        }

                        // Related functions
                        if let Some(Value::Vector(related)) =
                            m.get(&MapKey::Keyword(intern::intern_keyword("related")))
                        {
                            println!("\n{}", "Related:".cyan());
                            let related_names: Vec<String> = related
                                .iter()
                                .filter_map(|v| {
                                    if let Value::String(s) = v {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            println!("  {}", related_names.join(", ").bright_black());
                        }

                        println!();
                    }
                    _ => {
                        println!("\n{}: {}", name, ui_msg(UiMsg::ReplDocNoDoc));

                        // 類似の関数名を提案
                        let mut similar: Vec<String> = env
                            .bindings()
                            .map(|(n, _)| n.to_string())
                            .filter(|n| {
                                // 編集距離が2以下、または前方一致
                                strsim::damerau_levenshtein(name, n) <= 2 || n.starts_with(name)
                            })
                            .take(5)
                            .collect();

                        if !similar.is_empty() {
                            similar.sort();
                            println!("\n{}", "Did you mean:".bright_black());
                            for s in similar {
                                println!("  {}", s.yellow());
                            }
                        }
                        println!();
                    }
                }
            }
        }
        ":builtins" => {
            if let Some(env) = evaluator.get_env() {
                let env = env.read();
                let mut builtins: Vec<_> = env
                    .bindings()
                    .filter(|(_, v)| matches!(v, Value::NativeFunc(_)))
                    .map(|(name, _)| name.clone())
                    .collect();
                builtins.sort();

                if parts.len() > 1 {
                    // フィルタリング
                    let filter = parts[1];
                    builtins.retain(|name| name.contains(filter));

                    if builtins.is_empty() {
                        println!("{}", fmt_ui_msg(UiMsg::ReplNoBuiltinsMatching, &[filter]));
                    } else {
                        println!("{}", fmt_ui_msg(UiMsg::ReplBuiltinsMatching, &[filter]));
                        for (i, name) in builtins.iter().enumerate() {
                            if i % 4 == 0 {
                                print!("\n  ");
                            }
                            print!("{:<20}", name);
                        }
                        println!(
                            "\n\n{}",
                            fmt_ui_msg(UiMsg::ReplBuiltinTotal, &[&builtins.len().to_string()])
                        );
                    }
                } else {
                    // 全表示
                    println!("{}", ui_msg(UiMsg::ReplBuiltinFunctions));
                    for (i, name) in builtins.iter().enumerate() {
                        if i % 4 == 0 {
                            print!("\n  ");
                        }
                        print!("{:<20}", name);
                    }
                    println!(
                        "\n\n{}",
                        fmt_ui_msg(UiMsg::ReplBuiltinTotal, &[&builtins.len().to_string()])
                    );
                    println!("\n{}", ui_msg(UiMsg::ReplBuiltinTip));
                }
            }
        }
        ":clear" => {
            println!("{}", ui_msg(UiMsg::ReplEnvCleared));
        }
        ":load" => {
            if parts.len() < 2 {
                eprintln!("{}", ui_msg(UiMsg::ReplLoadUsage));
                return;
            }

            let path = parts[1];
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    println!("{}", fmt_ui_msg(UiMsg::ReplLoading, &[path]));
                    eval_repl_code(evaluator, &content, Some(path));
                    println!("{}", ui_msg(UiMsg::ReplLoaded));
                    *last_loaded_file = Some(path.to_string());
                }
                Err(e) => {
                    eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
                }
            }
        }
        ":reload" => {
            if let Some(path) = last_loaded_file.as_ref() {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        println!("{}", fmt_ui_msg(UiMsg::ReplLoading, &[path]));
                        eval_repl_code(evaluator, &content, Some(path));
                        println!("{}", ui_msg(UiMsg::ReplLoaded));
                    }
                    Err(e) => {
                        eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
                    }
                }
            } else {
                eprintln!("{}", ui_msg(UiMsg::ReplNoFileLoaded));
            }
        }
        ":watch" => {
            #[cfg(feature = "repl")]
            {
                if parts.len() < 2 {
                    eprintln!("Usage: :watch <file>");
                    return;
                }

                let file_path = parts[1];
                let abs_path = std::fs::canonicalize(file_path)
                    .unwrap_or_else(|_| std::path::PathBuf::from(file_path));
                let path_str = abs_path.to_string_lossy().to_string();

                // 既に監視中かチェック
                if watched_files.lock().contains(&path_str) {
                    println!("{}", format!("Already watching: {}", file_path).yellow());
                    return;
                }

                // ファイル監視を開始
                let tx = watch_tx.clone();
                let watch_path = abs_path.clone();

                // スレッド終了シグナル用のチャンネル
                let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

                std::thread::spawn(move || {
                    let (tx_event, rx_event) = std::sync::mpsc::channel();
                    let mut watcher = notify::recommended_watcher(
                        move |res: Result<notify::Event, notify::Error>| {
                            if let Ok(event) = res {
                                if matches!(event.kind, notify::EventKind::Modify(_)) {
                                    let _ = tx_event.send(());
                                }
                            }
                        },
                    )
                    .expect("Failed to create watcher");

                    watcher
                        .watch(&watch_path, RecursiveMode::NonRecursive)
                        .expect("Failed to watch file");

                    // イベントまたは終了シグナルを待つ
                    loop {
                        match (rx_event.try_recv(), stop_rx.try_recv()) {
                            (Ok(()), _) => {
                                // ファイル変更イベント
                                let _ = tx.send(watch_path.to_string_lossy().to_string());
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                            (_, Ok(())) => {
                                // 終了シグナル受信
                                break;
                            }
                            _ => {
                                // 何も受信していない場合はスリープして待つ
                                std::thread::sleep(std::time::Duration::from_millis(50));
                            }
                        }
                    }
                });

                watched_files.lock().insert(path_str.clone());
                watch_threads.lock().insert(path_str.clone(), stop_tx);
                println!("{}", format!("Watching: {}", file_path).green());
            }
            #[cfg(not(feature = "repl"))]
            {
                eprintln!("Watch feature is not available");
            }
        }
        ":unwatch" => {
            #[cfg(feature = "repl")]
            {
                if parts.len() < 2 {
                    // 全ての監視を停止
                    let paths: Vec<String> = watched_files.lock().iter().cloned().collect();
                    let count = paths.len();

                    for path in paths {
                        // スレッドに終了シグナルを送信
                        if let Some(stop_tx) = watch_threads.lock().remove(&path) {
                            let _ = stop_tx.send(());
                        }
                        watched_files.lock().remove(&path);
                    }

                    println!("{}", format!("Stopped watching {} file(s)", count).yellow());
                } else {
                    let file_path = parts[1];
                    let abs_path = std::fs::canonicalize(file_path)
                        .unwrap_or_else(|_| std::path::PathBuf::from(file_path));
                    let path_str = abs_path.to_string_lossy().to_string();

                    if watched_files.lock().remove(&path_str) {
                        // スレッドに終了シグナルを送信
                        if let Some(stop_tx) = watch_threads.lock().remove(&path_str) {
                            let _ = stop_tx.send(());
                        }
                        println!("{}", format!("Stopped watching: {}", file_path).yellow());
                    } else {
                        eprintln!("{}", format!("Not watching: {}", file_path).red());
                    }
                }
            }
            #[cfg(not(feature = "repl"))]
            {
                eprintln!("Watch feature is not available");
            }
        }
        ":threads" => {
            // Rayon thread pool情報を表示
            println!("Rayon Thread Pool:");
            println!("  Available parallelism: {}", rayon::current_num_threads());

            // チャンネル型の変数を探してステータス表示
            if let Some(env) = evaluator.get_env() {
                let env = env.read();
                let channels: Vec<_> = env
                    .bindings()
                    .filter(|(_, v)| matches!(v, Value::Channel(_)))
                    .collect();

                if !channels.is_empty() {
                    println!("\nActive Channels:");
                    for (name, value) in channels {
                        if let Value::Channel(ch) = value {
                            println!(
                                "  {} - len: {}, is_empty: {}",
                                name,
                                ch.receiver.len(),
                                ch.receiver.is_empty()
                            );
                        }
                    }
                }
            }
        }
        ":macro" => {
            #[cfg(feature = "repl")]
            {
                if parts.len() < 2 {
                    // マクロ一覧を表示
                    let list = macros.list();
                    if list.is_empty() {
                        println!("{}", "No macros defined".yellow());
                    } else {
                        println!("{}", "Defined macros:".cyan());
                        for (name, cmd) in list {
                            println!("  {} => {}", name.green(), cmd.bright_black());
                        }
                    }
                    return;
                }

                let subcommand = parts[1];
                match subcommand {
                    "define" | "def" => {
                        if parts.len() < 4 {
                            eprintln!("Usage: :macro define <name> <command>");
                            return;
                        }
                        let name = parts[2].to_string();
                        let command = parts[3..].join(" ");
                        macros.define(name.clone(), command.clone());
                        println!(
                            "{}",
                            format!("Macro '{}' defined: {}", name, command).green()
                        );
                    }
                    "delete" | "del" => {
                        if parts.len() < 3 {
                            eprintln!("Usage: :macro delete <name>");
                            return;
                        }
                        let name = parts[2];
                        if macros.delete(name) {
                            println!("{}", format!("Macro '{}' deleted", name).yellow());
                        } else {
                            eprintln!("{}", format!("Macro '{}' not found", name).red());
                        }
                    }
                    "list" => {
                        let list = macros.list();
                        if list.is_empty() {
                            println!("{}", "No macros defined".yellow());
                        } else {
                            println!("{}", "Defined macros:".cyan());
                            for (name, cmd) in list {
                                println!("  {} => {}", name.green(), cmd.bright_black());
                            }
                        }
                    }
                    _ => {
                        eprintln!("Unknown macro subcommand: {}", subcommand);
                        eprintln!("Available: define, delete, list");
                    }
                }
            }
            #[cfg(not(feature = "repl"))]
            {
                eprintln!("Macro feature is not available");
            }
        }
        ":m" => {
            // マクロ実行（短縮コマンド）
            #[cfg(feature = "repl")]
            {
                if parts.len() < 2 {
                    eprintln!("Usage: :m <macro-name>");
                    return;
                }
                let macro_name = parts[1];
                if let Some(command) = macros.get(macro_name) {
                    println!(
                        "{}",
                        format!("[Running macro '{}': {}]", macro_name, command).bright_black()
                    );
                    // TODO: マクロコマンドを実行
                    // ここでは単純に表示のみ（実行は別途実装が必要）
                    println!(
                        "{}",
                        "Note: Macro execution not yet implemented"
                            .to_string()
                            .yellow()
                    );
                } else {
                    eprintln!("{}", format!("Macro '{}' not found", macro_name).red());
                }
            }
            #[cfg(not(feature = "repl"))]
            {
                eprintln!("Macro feature is not available");
            }
        }
        ":profile" => {
            #[cfg(feature = "repl")]
            {
                if parts.len() < 2 {
                    eprintln!("Usage: :profile <start|stop|report|clear>");
                    return;
                }

                let subcommand = parts[1];
                match subcommand {
                    "start" => {
                        profile.start();
                        println!("{}", "Profiling started".green());
                    }
                    "stop" => {
                        profile.stop();
                        println!("{}", "Profiling stopped".yellow());
                    }
                    "report" => {
                        profile.report();
                    }
                    "clear" => {
                        profile.clear();
                        println!("{}", "Profiling data cleared".yellow());
                    }
                    _ => {
                        eprintln!("Unknown profile subcommand: {}", subcommand);
                        eprintln!("Available: start, stop, report, clear");
                    }
                }
            }
            #[cfg(not(feature = "repl"))]
            {
                eprintln!("Profile feature is not available");
            }
        }
        ":test" => {
            if parts.len() < 2 {
                // 引数なし: tests/ ディレクトリの全テストを実行
                // test/run-all を呼び出す
                println!("{}", "Running all tests...".cyan());
                eval_repl_code(evaluator, "(test/run-all)", None);
            } else {
                // ファイル指定: そのファイルを読み込んでtest/run-allを実行
                let path = parts[1];
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        println!("{}", format!("Running tests in {}...", path).cyan());
                        eval_repl_code(evaluator, &content, Some(path));
                        // テストを実行
                        eval_repl_code(evaluator, "(test/run-all)", None);
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Failed to read test file".red(), e);
                    }
                }
            }
        }
        ":trace" => {
            if parts.len() < 2 {
                // 引数なし: トレース中の関数一覧を表示
                let traced = qi_lang::builtins::debug::TRACED_FUNCTIONS.read();
                if traced.is_empty() {
                    println!("{}", "No functions are being traced".yellow());
                } else {
                    println!("{}", "Traced functions:".cyan());
                    for func in traced.iter() {
                        println!("  - {}", func);
                    }
                }
            } else {
                // 関数名を指定: トレース対象に追加
                let func_name = parts[1];
                qi_lang::builtins::debug::TRACED_FUNCTIONS
                    .write()
                    .insert(func_name.to_string());
                println!("{}", format!("Tracing function: {}", func_name).green());
            }
        }
        ":untrace" => {
            if parts.len() < 2 {
                // 引数なし: 全トレースを停止
                qi_lang::builtins::debug::TRACED_FUNCTIONS.write().clear();
                println!("{}", "Stopped tracing all functions".yellow());
            } else {
                // 関数名を指定: トレース対象から削除
                let func_name = parts[1];
                let removed = qi_lang::builtins::debug::TRACED_FUNCTIONS
                    .write()
                    .remove(func_name);
                if removed {
                    println!("{}", format!("Stopped tracing: {}", func_name).yellow());
                } else {
                    println!("{}", format!("Function not traced: {}", func_name).red());
                }
            }
        }
        ":quit" => {
            // handled in main loop
        }
        _ => {
            eprintln!("{}", fmt_ui_msg(UiMsg::ReplUnknownCommand, &[command]));
            eprintln!("{}", ui_msg(UiMsg::ReplTypeHelpForCommands));
        }
    }
}

/// REPL用のコード評価（エラーで終了しない）
fn eval_repl_code(evaluator: &Evaluator, code: &str, filename: Option<&str>) {
    // ソース情報を設定
    let source_name = filename.unwrap_or("<repl>").to_string();
    evaluator.set_source(source_name.clone(), code.to_string());

    match Parser::new(code) {
        Ok(mut parser) => {
            parser.set_source_name(source_name);
            match parser.parse_all() {
                Ok(exprs) => {
                    for expr in exprs.iter() {
                        match evaluator.eval(expr) {
                            Ok(_) => {}
                            Err(e) => {
                                print_error(ui_msg(UiMsg::ErrorRuntime), &e);
                            }
                        }
                    }
                }
                Err(e) => {
                    print_error(ui_msg(UiMsg::ErrorParse), &e);
                }
            }
        }
        Err(e) => {
            print_error(ui_msg(UiMsg::ErrorLexer), &e);
        }
    }
}
