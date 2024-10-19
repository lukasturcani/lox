#![warn(rust_2018_idioms)]

use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    file: Option<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    match cli.file {
        Some(file) => run_file(file),
        None => run_prompt(),
    }
}

fn run_file(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let content = std::fs::read(path)?;
    run(&content)
}

fn run_prompt() -> Result<(), anyhow::Error> {
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        if line.is_empty() {
            break;
        }
        run(line.as_bytes())?;
    }
    Ok(())
}

fn run(content: &[u8]) -> Result<(), anyhow::Error> {
    let tokens = scan(content);
    for token in tokens {
        println!("{token:?}")
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum TokenType {
    EndOfFile,
}

#[derive(Debug, Clone)]
struct Token {
    r#type: TokenType,
    line: usize,
}

struct Scanner {
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new() -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

fn scan(content: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new();
    while scanner.current >= content.len() {
        scanner.start = scanner.current;
        scanner.scan_token();
    }
    tokens.push(Token {
        r#type: TokenType::EndOfFile,
        line: scanner.line,
    })
}

fn error(line: usize, message: &str) {
    report(line, "", message)
}

fn report(line: usize, r#where: &str, message: &str) {
    println!("[line {line}] Error {where}: {message}")
}
