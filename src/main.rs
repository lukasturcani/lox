#![warn(rust_2018_idioms)]
#![feature(portable_simd)]

use std::{io::Write, path::PathBuf, string::FromUtf8Error};

use anyhow::anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    file: Option<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    if let Some(file) = args.file {
        run(&std::fs::read(file)?)
    } else {
        run_prompt()
    }
}

fn run(source: &[u8]) -> Result<(), anyhow::Error> {
    match scan_tokens(source) {
        Ok(tokens) => {
            for token in tokens {
                println!("{token:#?}");
            }
            Ok(())
        }
        Err(errors) => Err(anyhow!("errors {errors:#?}")),
    }
}

fn run_prompt() -> Result<(), anyhow::Error> {
    let mut line = String::new();
    print!("> ");
    std::io::stdout().flush()?;
    while let Ok(n) = std::io::stdin().read_line(&mut line) {
        if n == 0 {
            break;
        }
        run(line.as_bytes())?;
        print!("> ");
        std::io::stdout().flush()?;
        line.clear();
    }
    Ok(())
}

#[derive(Debug)]
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
    Slash,
    Star,
    Not,
    NotEqual,
    Equal,
    Assign,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Identifier,
    String(String),
    Number,
    And,
    Class,
    Else,
    False,
    Function,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Debug)]
struct Token {
    line: usize,
    r#type: TokenType,
}

#[derive(Debug)]
struct ScanErrors {
    errors: Vec<ScanError>,
}

#[derive(Debug)]
enum ScanError {
    UnexpectedCharacter { character: char, line: usize },
    UnterminatedString,
    InvalidString(FromUtf8Error),
}

fn scan_tokens(source: &[u8]) -> Result<Vec<Token>, ScanErrors> {
    Scanner::default().scan_tokens(source)
}

#[derive(Debug, Default)]
struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<ScanError>,
}

impl Scanner {
    fn scan_tokens(mut self, source: &[u8]) -> Result<Vec<Token>, ScanErrors> {
        self.start = 0;
        self.current = 1;
        self.line = 1;
        while self.current < source.len() {
            let chars = &source[self.start..self.current];
            match chars {
                b"(" => self.add_token(TokenType::LeftBracket),
                b")" => self.add_token(TokenType::RightBracket),
                b"{" => self.add_token(TokenType::LeftBrace),
                b"}" => self.add_token(TokenType::RightBrace),
                b"," => self.add_token(TokenType::Comma),
                b"." => self.add_token(TokenType::Dot),
                b"-" => self.add_token(TokenType::Minus),
                b"+" => self.add_token(TokenType::Plus),
                b";" => self.add_token(TokenType::Semicolon),
                b"*" => self.add_token(TokenType::Star),
                b"!" => {
                    if source[self.current] == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::NotEqual)
                    } else {
                        self.add_token(TokenType::Not)
                    }
                }
                b"=" => {
                    if source[self.current] == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::Equal)
                    } else {
                        self.add_token(TokenType::Assign)
                    }
                }
                b"<" => {
                    if source[self.current] == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::LessThanOrEqual)
                    } else {
                        self.add_token(TokenType::LessThan)
                    }
                }
                b">" => {
                    if source[self.current] == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::GreaterThanOrEqual)
                    } else {
                        self.add_token(TokenType::GreaterThan)
                    }
                }
                b"/" => {
                    if source[self.current] == b'/' {
                        self.current += 1;
                        while self.current < source.len() && source[self.current] != b'\n' {
                            self.current += 1;
                        }
                        self.start = self.current;
                        self.current += 1;
                    } else {
                        self.add_token(TokenType::Slash)
                    }
                }
                b" " | b"\t" | b"\r" => {
                    self.start = self.current;
                    self.current += 1;
                }
                b"\n" => {
                    self.start = self.current;
                    self.current += 1;
                    self.line += 1;
                }
                b"\"" => {
                    while self.current < source.len() && source[self.current] != b'\"' {
                        if source[self.current - 1] == b'\n' {
                            self.line += 1;
                        }
                        self.current += 1;
                    }
                    if self.current == source.len() {
                        self.errors.push(ScanError::UnterminatedString);
                    } else {
                        self.current += 1;
                        match String::from_utf8(source[self.start + 1..self.current].into()) {
                            Ok(literal) => {
                                self.add_token(TokenType::String(literal));
                            }
                            Err(error) => self.errors.push(ScanError::InvalidString(error)),
                        }
                    }
                }
                _ => {
                    self.errors.push(ScanError::UnexpectedCharacter {
                        character: source[self.current].into(),
                        line: self.line,
                    });
                    self.start = self.current;
                    self.current += 1;
                }
            }
        }
        if !self.errors.is_empty() {
            Err(ScanErrors {
                errors: self.errors,
            })
        } else {
            Ok(self.tokens)
        }
    }
    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token {
            r#type: token,
            line: self.line,
        });
        self.start = self.current;
        self.current += 1;
    }
}

fn print_error(line: usize, message: &str) {
    report_error(line, "", message);
}

fn report_error(line: usize, r#where: &str, message: &str) {
    eprintln!("[line {line}] Error {where}: {message}")
}
