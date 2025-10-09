use qi_lang::eval::Evaluator;
use qi_lang::i18n;
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
                eprintln!("Error: {} requires an argument", args[1]);
                std::process::exit(1);
            }
            run_code(&args[2]);
        }
        "-l" | "--load" => {
            // REPLでファイルをロード
            if args.len() < 3 {
                eprintln!("Error: {} requires a file path", args[1]);
                std::process::exit(1);
            }
            repl(Some(&args[2]));
        }
        arg if arg.starts_with('-') => {
            eprintln!("Error: Unknown option: {}", arg);
            eprintln!("Use --help for usage information");
            std::process::exit(1);
        }
        _ => {
            // ファイルを実行
            run_file(&args[1]);
        }
    }
}

fn print_help() {
    println!("Qi - A Lisp that flows");
    println!();
    println!("USAGE:");
    println!("    qi [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -e, -c <code>       Execute code string and exit");
    println!("    -l, --load <file>   Load file and start REPL");
    println!("    -h, --help          Print help information");
    println!("    -v, --version       Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    qi                       Start REPL");
    println!("    qi script.qi             Run script file");
    println!("    qi -e '(+ 1 2 3)'        Execute code and print result");
    println!("    qi -l utils.qi           Load file and start REPL");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    QI_LANG              Set language (ja, en)");
    println!("    LANG                 System locale (auto-detected)");
}

fn run_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error: Failed to read file: {}", e);
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
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    }
}

fn repl(preload: Option<&str>) {
    println!("Qi REPL v{}", VERSION);
    println!("Press Ctrl+C to exit");
    println!();

    let mut evaluator = Evaluator::new();

    // ファイルのプリロード
    if let Some(path) = preload {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                println!("Loading {}...", path);
                eval_code(&mut evaluator, &content, false);
                println!("Loaded.\n");
            }
            Err(e) => {
                eprintln!("Error: Failed to load file: {}", e);
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
                            Err(e) => eprintln!("Error: {}", e),
                        },
                        Err(e) => eprintln!("Parse error: {}", e),
                    },
                    Err(e) => eprintln!("Lexer error: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }

    println!("\nGoodbye!");
}
