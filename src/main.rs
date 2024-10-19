#![warn(rust_2018_idioms)]

use std::{
    io::Write,
    path::{Path, PathBuf},
};

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
    let mut stdout = std::io::stdout();
    let mut line = String::new();
    loop {
        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut line)?;
        if line.is_empty() {
            break;
        }
        run(line.as_bytes())?;
        line.clear();
    }
    Ok(())
}

fn run(content: &[u8]) -> Result<(), anyhow::Error> {
    let tokens = scan_tokens(content).map_err(|error| anyhow::anyhow!("errors {error:?}"))?;
    for token in tokens {
        println!("{token:?}")
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    EndOfFile,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    r#type: TokenType,
    line: usize,
}

#[derive(Debug, PartialEq)]
enum ScanError {
    UnexpectedCharacter { character: char, line: usize },
}

#[derive(Debug, PartialEq)]
struct ScanErrors {
    errors: Vec<ScanError>,
}

fn scan_tokens(content: &[u8]) -> Result<Vec<Token>, ScanErrors> {
    Scanner::new().scan_tokens(content)
}

struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    errors: Vec<ScanError>,
    tokens: Vec<Token>,
}

impl Scanner {
    fn new() -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
            errors: Vec::new(),
            tokens: Vec::new(),
        }
    }

    fn scan_tokens(mut self, content: &[u8]) -> Result<Vec<Token>, ScanErrors> {
        while self.current < content.len() {
            match content[self.current] {
                b'(' => self.add_token(TokenType::LeftBracket),
                b')' => self.add_token(TokenType::RightBracket),
                b'{' => self.add_token(TokenType::LeftBrace),
                b'}' => self.add_token(TokenType::RightBrace),
                b',' => self.add_token(TokenType::Comma),
                b'.' => self.add_token(TokenType::Dot),
                b'-' => self.add_token(TokenType::Minus),
                b'+' => self.add_token(TokenType::Plus),
                b';' => self.add_token(TokenType::Semicolon),
                b'*' => self.add_token(TokenType::Star),
                unexpected => {
                    self.errors.push(ScanError::UnexpectedCharacter {
                        character: unexpected as char,
                        line: self.line,
                    });
                    self.start = self.current;
                    self.current += 1;
                }
            }
        }
        self.tokens.push(Token {
            r#type: TokenType::EndOfFile,
            line: self.line,
        });
        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(ScanErrors {
                errors: self.errors,
            })
        }
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.tokens.push(Token {
            r#type,
            line: self.line,
        });
        self.start = self.current;
        self.current += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        let tokens = scan_tokens(b"(){}");
        assert_eq!(
            tokens,
            Ok(vec![
                Token {
                    line: 1,
                    r#type: TokenType::LeftBracket,
                },
                Token {
                    line: 1,
                    r#type: TokenType::RightBracket,
                },
                Token {
                    line: 1,
                    r#type: TokenType::LeftBrace,
                },
                Token {
                    line: 1,
                    r#type: TokenType::RightBrace
                },
                Token {
                    line: 1,
                    r#type: TokenType::EndOfFile,
                },
            ])
        )
    }
}
