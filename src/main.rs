#![warn(rust_2018_idioms)]

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;

mod interpreter;
mod parser;
mod scanner;

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
    let mut stdout = std::io::stdout();
    let mut line = String::new();
    loop {
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut line)?;
        if line.is_empty() {
            break;
        }
        if let Err(errors) = run(line.as_bytes()) {
            println!("{errors:?}")
        }
        line.clear();
    }
    Ok(())
}

fn run(content: &[u8]) -> Result<(), anyhow::Error> {
    let tokens = scanner::scan_tokens(content).map_err(|error| anyhow::anyhow!("{error:?}"))?;
    let expr = parser::parse(tokens).map_err(|error| anyhow::anyhow!("{error:?}"))?;
    let value = interpreter::interpret(&expr).map_err(|error| anyhow::anyhow!("{error:?}"))?;
    println!("{value:?}");
    Ok(())
}
