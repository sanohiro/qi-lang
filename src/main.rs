use qi_lang::eval::Evaluator;
use qi_lang::i18n::{self, fmt_ui_msg, ui_msg, UiMsg};
use qi_lang::parser::Parser;
use qi_lang::value::Value;
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

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        completions.insert(":quit".to_string());

        QiHelper { completions }
    }

    fn update_completions(&mut self, evaluator: &Evaluator) {
        // 環境から変数と関数を取得
        if let Some(env) = evaluator.get_env() {
            let env = env.read();
            for (name, _) in env.bindings() {
                self.completions.insert(name.clone());
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

impl Highlighter for QiHelper {}
impl Hinter for QiHelper {
    type Hint = String;
}
impl Validator for QiHelper {}
impl Helper for QiHelper {}

fn main() {
    // 国際化システムを初期化
    i18n::init();

    let args: Vec<String> = std::env::args().collect();

    // コマンドライン引数の解析
    if args.len() == 1 {
        // 引数なし: REPLを起動
        repl(None);
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        }
        "-v" | "--version" => {
            println!("{}", fmt_ui_msg(UiMsg::VersionString, &[VERSION]));
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
            repl(Some(&args[2]));
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
    println!("{}:", ui_msg(UiMsg::HelpUsage));
    println!("    qi [OPTIONS] [FILE]");
    println!();
    println!("{}:", ui_msg(UiMsg::HelpOptions));
    println!("    -e, -c <code>       {}", ui_msg(UiMsg::OptExecute));
    println!("    -                   {}", ui_msg(UiMsg::OptStdin));
    println!("    -l, --load <file>   {}", ui_msg(UiMsg::OptLoad));
    println!("    -h, --help          {}", ui_msg(UiMsg::OptHelp));
    println!("    -v, --version       {}", ui_msg(UiMsg::OptVersion));
    println!();
    println!("{}:", ui_msg(UiMsg::HelpExamples));
    println!(
        "    qi                       {}",
        ui_msg(UiMsg::ExampleStartRepl)
    );
    println!(
        "    qi script.qi             {}",
        ui_msg(UiMsg::ExampleRunScript)
    );
    println!(
        "    qi -e '(+ 1 2 3)'        {}",
        ui_msg(UiMsg::ExampleExecuteCode)
    );
    println!(
        "    echo '(println 42)' | qi -    {}",
        ui_msg(UiMsg::ExampleStdin)
    );
    println!(
        "    qi -l utils.qi           {}",
        ui_msg(UiMsg::ExampleLoadFile)
    );
    println!();
    println!("{}:", ui_msg(UiMsg::HelpEnvVars));
    println!("    QI_LANG              {}", ui_msg(UiMsg::EnvLangQi));
    println!("    LANG                 {}", ui_msg(UiMsg::EnvLangSystem));
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
        match Parser::new(&content) {
            Ok(mut parser) => match parser.parse_all() {
                Ok(exprs) => {
                    for expr in exprs.iter() {
                        if let Err(e) = evaluator.eval(expr) {
                            eprintln!("{}:{}: {}", test_file, ui_msg(UiMsg::ErrorRuntime), e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}:{}: {}", test_file, ui_msg(UiMsg::ErrorParse), e);
                }
            },
            Err(e) => {
                eprintln!("{}:{}: {}", test_file, ui_msg(UiMsg::ErrorLexer), e);
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
                eprintln!("Internal error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Internal error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_code(code: &str) {
    let mut evaluator = Evaluator::new();
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
    match Parser::new(code) {
        Ok(mut parser) => match parser.parse_all() {
            Ok(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    match evaluator.eval(expr) {
                        Ok(value) => {
                            // ワンライナーの場合、最後の結果を表示
                            if print_result && i == exprs.len() - 1 {
                                println!("{}", value);
                            }
                        }
                        Err(e) => {
                            if let Some(file) = filename {
                                eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorRuntime), e);
                            } else {
                                eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e);
                            }
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(file) = filename {
                    eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorParse), e);
                } else {
                    eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e);
                }
                std::process::exit(1);
            }
        },
        Err(e) => {
            if let Some(file) = filename {
                eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorLexer), e);
            } else {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e);
            }
            std::process::exit(1);
        }
    }
}

fn repl(preload: Option<&str>) {
    println!("{}", fmt_ui_msg(UiMsg::ReplWelcome, &[VERSION]));
    println!("{}", ui_msg(UiMsg::ReplPressCtrlC));
    println!("{}", ui_msg(UiMsg::ReplTypeHelp));
    println!();

    let mut evaluator = Evaluator::new();
    let mut helper = QiHelper::new();
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(helper));

    let history_file = dirs::home_dir()
        .map(|p| p.join(".qi_history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".qi_history"));

    let _ = rl.load_history(&history_file);

    let mut last_loaded_file: Option<String> = None;

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
    let mut accumulated_input = String::new();

    loop {
        let prompt = if accumulated_input.is_empty() {
            format!("qi:{}> ", line_number)
        } else {
            "     ... ".to_string()
        };

        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // REPLコマンドの処理
                if line.starts_with(':') && accumulated_input.is_empty() {
                    rl.add_history_entry(line).ok();
                    handle_repl_command(line, &evaluator, &mut last_loaded_file);

                    // :clear の場合は evaluator と helper をリセット
                    if line == ":clear" {
                        evaluator = Evaluator::new();
                        helper = QiHelper::new();
                        rl.set_helper(Some(helper));
                    } else if line == ":quit" {
                        break;
                    } else {
                        // 補完候補を更新
                        if let Some(h) = rl.helper_mut() {
                            h.update_completions(&evaluator);
                        }
                    }
                    continue;
                }

                // 複数行入力の処理
                if !accumulated_input.is_empty() {
                    accumulated_input.push('\n');
                }
                accumulated_input.push_str(line);

                // 括弧のバランスチェック
                if !is_balanced(&accumulated_input) {
                    continue;
                }

                rl.add_history_entry(&accumulated_input).ok();

                match Parser::new(&accumulated_input) {
                    Ok(mut parser) => match parser.parse() {
                        Ok(expr) => match evaluator.eval(&expr) {
                            Ok(value) => {
                                println!("{}", value);
                                line_number += 1;

                                // 補完候補を更新
                                if let Some(h) = rl.helper_mut() {
                                    h.update_completions(&evaluator);
                                }
                            }
                            Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e),
                        },
                        Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e),
                    },
                    Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e),
                }

                accumulated_input.clear();
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                accumulated_input.clear();
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
    println!("\n{}", ui_msg(UiMsg::ReplGoodbye));
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
fn handle_repl_command(cmd: &str, evaluator: &Evaluator, last_loaded_file: &mut Option<String>) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0];

    match command {
        ":help" => {
            println!("{}", ui_msg(UiMsg::ReplAvailableCommands));
            println!("  {}", ui_msg(UiMsg::ReplCommandHelp));
            println!("  :doc <name>              Show documentation for a function");
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
                eprintln!("Usage: :doc <name>");
                return;
            }

            let name = parts[1];
            if let Some(env) = evaluator.get_env() {
                let env = env.read();

                // 関数/変数の存在確認
                if env.get(name).is_none() {
                    eprintln!("No such function or variable: {}", name);
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
                        println!("\n{}: {}\n", name, s);
                    }
                    Some(Value::Map(m)) => {
                        // 構造化ドキュメント
                        println!("\n{}:", name);
                        if let Some(Value::String(desc)) = m.get("desc") {
                            println!("  {}", desc);
                        }
                        if let Some(Value::Vector(params)) = m.get("params") {
                            println!("\nParameters:");
                            for param in params {
                                if let Value::Map(pm) = param {
                                    if let (
                                        Some(Value::String(pname)),
                                        Some(Value::String(pdesc)),
                                    ) = (pm.get("name"), pm.get("desc"))
                                    {
                                        println!("  {} - {}", pname, pdesc);
                                    }
                                }
                            }
                        }
                        if let Some(Value::Vector(examples)) = m.get("examples") {
                            println!("\nExamples:");
                            for ex in examples {
                                if let Value::String(s) = ex {
                                    println!("  {}", s);
                                }
                            }
                        }
                        println!();
                    }
                    _ => {
                        println!("\n{}: (no documentation available)\n", name);
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
    match Parser::new(code) {
        Ok(mut parser) => match parser.parse_all() {
            Ok(exprs) => {
                for expr in exprs.iter() {
                    match evaluator.eval(expr) {
                        Ok(_) => {}
                        Err(e) => {
                            if let Some(file) = filename {
                                eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorRuntime), e);
                            } else {
                                eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(file) = filename {
                    eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorParse), e);
                } else {
                    eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e);
                }
            }
        },
        Err(e) => {
            if let Some(file) = filename {
                eprintln!("{}:{}: {}", file, ui_msg(UiMsg::ErrorLexer), e);
            } else {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e);
            }
        }
    }
}
