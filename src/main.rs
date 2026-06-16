mod ast;
mod value;
mod environment;
mod evaluator;
mod gui;

include!(concat!(env!("OUT_DIR"), "/grammar.rs"));

use std::env;
use std::fs;
use std::io::{self, Write};
use anyhow::Result;
use environment::Environment;
use evaluator::Evaluator;
use gui::ComixApp;
use eframe::egui;


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--gui" {
        run_gui();
        Ok(())
    } else {
        // 原有 REPL 和文件逻辑
        match args.len() {
            1 => run_repl(),
            2 => run_file(&args[1]),
            _ => {
                eprintln!("用法: comix [脚本文件] 或 comix --gui");
                std::process::exit(64);
            }
        }
    }
}

fn run_gui() {
    use eframe::{NativeOptions, CreationContext};
    let env = Environment::new();
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Comix IDE",
        options,
        Box::new(move |_cc: &CreationContext<'_>| {
            Ok(Box::new(ComixApp::new(env, String::new())) as Box<dyn eframe::App>)
        }),
    )
    .unwrap();
}

fn run_repl() -> Result<()> {
    println!("Comix语言解释器 v0.1.0");
    println!("输入 'exit' 退出, 'clear' 清除环境, 'dump' 打印变量");
    println!("多行输入：行尾使用 '\\' 续行");

    let env = Environment::new();
    let mut evaluator = Evaluator::new(env, String::new());

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim_end();

        if line.is_empty() {
            continue;
        }

        match line {
            "exit" => break,
            "clear" => {
                evaluator = Evaluator::new(Environment::new(), String::new());
                println!("环境已清除");
                continue;
            }
            "dump" => {
                evaluator.dump_env();
                continue;
            }
            _ => {}
        }

        // 多行输入缓冲区
        let mut buffer = String::new();
        let mut current_line = line.to_string(); // 拥有所有权的 String

        loop {
            if current_line.ends_with('\\') {
                // 去掉末尾的反斜杠
                current_line.pop();
                buffer.push_str(&current_line);
                buffer.push('\n');
                print!("... ");
                io::stdout().flush()?;
                let mut next_line = String::new();
                io::stdin().read_line(&mut next_line)?;
                let trimmed = next_line.trim_end();
                if trimmed.is_empty() {
                    // 空行结束输入
                    break;
                }
                current_line = trimmed.to_string();
            } else {
                buffer.push_str(&current_line);
                break;
            }
        }

        let source = buffer;
        evaluator.set_source(source.clone());

        match parse_input(&source) {
            Ok(ast) => match evaluator.evaluate(&ast) {
                Ok(value) => {
                    if !value.is_null() {
                        println!("{}", value);
                    }
                }
                Err(e) => eprintln!("运行时错误: {}", e),
            },
            Err(e) => eprintln!("解析错误: {}", e),
        }
    }

    Ok(())
}

fn get_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    if offset >= bytes.len() {
        return (1, 1);
    }
    let mut line = 1;
    let mut col = 1;
    for (byte_idx, byte) in source.as_bytes().iter().enumerate() {
        if byte_idx == offset {
            break;
        }
        if *byte == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn parse_input(input: &str) -> Result<Vec<ast::Stmt>, String> {
    use lalrpop_util::lalrpop_mod;
    use lalrpop_util::ParseError;
    lalrpop_mod!(pub grammar);

    let parser = grammar::StmtListParser::new();
    parser.parse(input).map_err(|e| match e {
        ParseError::InvalidToken { location } => {
            let (line, col) = get_line_col(input, location);
            format!("{}:{} 无效字符: '{}'", line, col, &input[location..location + 1])
        }
        ParseError::UnrecognizedEof { location, expected } => {
            let (line, col) = get_line_col(input, location);
            format!("{}:{} 未预期的文件结束, 期望: {:?}", line, col, expected)
        }
        ParseError::UnrecognizedToken { token, expected } => {
            let (start, end) = (token.0, token.2);
            let (line, col) = get_line_col(input, start);
            let token_text = &input[start..end];
            format!("{}:{} 未识别的 token '{}', 期望: {:?}", line, col, token_text, expected)
        }
        ParseError::ExtraToken { token } => {
            let (start, end) = (token.0, token.2);
            let (line, col) = get_line_col(input, start);
            let token_text = &input[start..end];
            format!("{}:{} 多余的 token '{}'", line, col, token_text)
        }
        ParseError::User { error } => format!("解析错误: {}", error),
    })
}

fn run_file(path: &str) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let source = source.trim_start_matches('\u{feff}').to_string();
    let env = Environment::new();
    let mut evaluator = Evaluator::new(env, source.clone());

    match parse_input(&source) {
        Ok(ast) => match evaluator.evaluate(&ast) {
            Ok(value) => {
                if !value.is_null() {
                    println!("{}", value);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("运行时错误: {}", e);
                std::process::exit(70);
            }
        },
        Err(e) => {
            eprintln!("解析错误: {}", e);
            std::process::exit(65);
        }
    }
}