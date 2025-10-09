use qi_lang::eval::Evaluator;
use qi_lang::i18n::{self, fmt_ui_msg, ui_msg, UiMsg};
use qi_lang::parser::Parser;
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            println!("Qi version {}", VERSION);
        }
        "-e" | "-c" => {
            // ワンライナー実行
            if args.len() < 3 {
                eprintln!("{}", fmt_ui_msg(UiMsg::ErrorRequiresArg, &[&args[1]]));
                std::process::exit(1);
            }
            run_code(&args[2]);
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
    println!("    -l, --load <file>   {}", ui_msg(UiMsg::OptLoad));
    println!("    -h, --help          {}", ui_msg(UiMsg::OptHelp));
    println!("    -v, --version       {}", ui_msg(UiMsg::OptVersion));
    println!();
    println!("{}:", ui_msg(UiMsg::HelpExamples));
    println!("    qi                       Start REPL");
    println!("    qi script.qi             Run script file");
    println!("    qi -e '(+ 1 2 3)'        Execute code and print result");
    println!("    qi -l utils.qi           Load file and start REPL");
    println!();
    println!("{}:", ui_msg(UiMsg::HelpEnvVars));
    println!("    QI_LANG              Set language (ja, en)");
    println!("    LANG                 System locale (auto-detected)");
}

fn run_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
        std::process::exit(1);
    });

    let mut evaluator = Evaluator::new();
    eval_code(&mut evaluator, &content, false);
}

fn run_code(code: &str) {
    let mut evaluator = Evaluator::new();
    eval_code(&mut evaluator, code, true);
}

fn eval_code(evaluator: &mut Evaluator, code: &str, print_result: bool) {
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
                            eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e);
            std::process::exit(1);
        }
    }
}

fn repl(preload: Option<&str>) {
    println!("{}", fmt_ui_msg(UiMsg::ReplWelcome, &[VERSION]));
    println!("{}", ui_msg(UiMsg::ReplPressCtrlC));
    println!();

    let mut evaluator = Evaluator::new();

    // ファイルのプリロード
    if let Some(path) = preload {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                println!("{}", fmt_ui_msg(UiMsg::ReplLoading, &[path]));
                eval_code(&mut evaluator, &content, false);
                println!("{}\n", ui_msg(UiMsg::ReplLoaded));
            }
            Err(e) => {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorFailedToRead), e);
                std::process::exit(1);
            }
        }
    }

    let mut line_number = 1;

    loop {
        print!("qi:{}> ", line_number);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match Parser::new(input) {
                    Ok(mut parser) => match parser.parse() {
                        Ok(expr) => match evaluator.eval(&expr) {
                            Ok(value) => {
                                println!("{}", value);
                                line_number += 1;
                            }
                            Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorRuntime), e),
                        },
                        Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorParse), e),
                    },
                    Err(e) => eprintln!("{}: {}", ui_msg(UiMsg::ErrorLexer), e),
                }
            }
            Err(e) => {
                eprintln!("{}: {}", ui_msg(UiMsg::ErrorInput), e);
                break;
            }
        }
    }

    println!("\n{}", ui_msg(UiMsg::ReplGoodbye));
}
