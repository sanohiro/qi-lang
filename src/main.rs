use qi_lang::eval::Evaluator;
use qi_lang::i18n;
use qi_lang::parser::Parser;
use std::io::{self, Write};

fn main() {
    // 国際化システムを初期化
    i18n::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // ファイルを実行
        run_file(&args[1]);
    } else {
        // REPLを起動
        repl();
    }
}

fn run_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("ファイルの読み込みに失敗しました: {}", e);
        std::process::exit(1);
    });

    let mut evaluator = Evaluator::new();

    match Parser::new(&content) {
        Ok(mut parser) => match parser.parse_all() {
            Ok(exprs) => {
                for expr in exprs {
                    match evaluator.eval(&expr) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("エラー: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("パースエラー: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("レキサーエラー: {}", e);
            std::process::exit(1);
        }
    }
}

fn repl() {
    println!("Qi REPL v0.1.0");
    println!("終了するには Ctrl+C を押してください");
    println!();

    let mut evaluator = Evaluator::new();
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
                            Err(e) => eprintln!("エラー: {}", e),
                        },
                        Err(e) => eprintln!("パースエラー: {}", e),
                    },
                    Err(e) => eprintln!("レキサーエラー: {}", e),
                }
            }
            Err(e) => {
                eprintln!("入力エラー: {}", e);
                break;
            }
        }
    }

    println!("\nさようなら!");
}
