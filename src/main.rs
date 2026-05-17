mod ast;
mod value;
mod environment;
mod evaluator;

include!(concat!(env!("OUT_DIR"), "/grammar.rs"));

use std::env;
use std::fs;
use std::io::{self, Write};
use anyhow::Result;

use environment::Environment;
use evaluator::Evaluator;
// use lalrpop_util::ParseError;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => run_repl(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("用法: comix [脚本文件]");
            std::process::exit(64);
        }
    }
}

/// 运行交互式REPL
fn run_repl() -> Result<()> {
    println!("Comix语言解释器 v0.1.0");
    println!("输入 'exit' 退出, 'clear' 清除环境");
    
    let env = Environment::new();
    let mut evaluator = Evaluator::new(env);
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        if line == "exit" {
            break;
        }
        
        if line == "clear" {
            evaluator = Evaluator::new(Environment::new());
            println!("环境已清除");
            continue;
        }
        
        match parse_input(line) {
            Ok(ast) => {
                match evaluator.evaluate(&ast) {
                    Ok(value) => {
                        if !value.is_null() {
                            println!("{}", value);
                        }
                    }
                    Err(e) => {
                        eprintln!("运行时错误: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("解析错误: {}", e);
            }
        }
    }
    
    Ok(())
}

/// 解析单行输入
fn parse_input(input: &str) -> Result<Vec<ast::Stmt>, String> {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub grammar);
    
    let parser = grammar::StmtListParser::new();
    parser.parse(input)
        .map_err(|e| format!("{:?}", e))
}

/// 运行文件
fn run_file(path: &str) -> Result<()> {
    let source = fs::read_to_string(path)?;
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub grammar);
    
    let parser = grammar::StmtListParser::new();
    let env = Environment::new();
    let mut evaluator = Evaluator::new(env);
    
    match parser.parse(&source) {
        Ok(ast) => {
            match evaluator.evaluate(&ast) {
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
            }
        }
        Err(e) => {
            eprintln!("解析错误: {:?}", e);
            std::process::exit(65);
        }
    }
}