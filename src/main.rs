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

#[derive(Debug, PartialEq)]
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
    Number(f32),
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

#[derive(Debug, PartialEq)]
struct Token {
    line: usize,
    r#type: TokenType,
}

#[derive(Debug, PartialEq)]
struct ScanErrors {
    errors: Vec<ScanError>,
}

#[derive(Debug, PartialEq)]
enum ScanError {
    UnexpectedCharacter { character: char, line: usize },
    UnterminatedString { line: usize },
    InvalidString { line: usize, error: FromUtf8Error },
    InvalidNumber { line: usize, number: String },
    InvalidUtf8 { line: usize, bytes: Vec<u8> },
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
        self.current = 0;
        self.line = 1;
        while self.current < source.len() {
            match source[self.current] {
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
                b'!' => {
                    if self.peek(source) == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::NotEqual)
                    } else {
                        self.add_token(TokenType::Not)
                    }
                }
                b'=' => {
                    if self.peek(source) == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::Equal)
                    } else {
                        self.add_token(TokenType::Assign)
                    }
                }
                b'<' => {
                    if self.peek(source) == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::LessThanOrEqual)
                    } else {
                        self.add_token(TokenType::LessThan)
                    }
                }
                b'>' => {
                    if self.peek(source) == b'=' {
                        self.current += 1;
                        self.add_token(TokenType::GreaterThanOrEqual)
                    } else {
                        self.add_token(TokenType::GreaterThan)
                    }
                }
                b'/' => {
                    if self.peek(source) == b'/' {
                        self.current += 1;
                        while self.current < source.len() && self.peek(source) != b'\n' {
                            self.current += 1;
                        }
                        self.current += 1;
                    } else {
                        self.add_token(TokenType::Slash)
                    }
                }
                b' ' | b'\t' | b'\r' => {
                    self.current += 1;
                    self.start = self.current;
                }
                b'\n' => {
                    self.current += 1;
                    self.start = self.current;
                    self.line += 1;
                }
                b'\"' => {
                    while self.current < source.len() && self.peek(source) != b'\"' {
                        if source[self.current] == b'\n' {
                            self.line += 1;
                        }
                        self.current += 1;
                    }
                    if self.current == source.len() {
                        self.errors
                            .push(ScanError::UnterminatedString { line: self.line });
                    } else {
                        self.current += 1;
                        match String::from_utf8(source[self.start + 1..self.current].into()) {
                            Ok(literal) => {
                                self.add_token(TokenType::String(literal));
                            }
                            Err(error) => self.errors.push(ScanError::InvalidString {
                                line: self.line,
                                error,
                            }),
                        }
                    }
                }
                digit if digit.is_ascii_digit() => {
                    while self.current < source.len() && source[self.current].is_ascii_digit() {
                        self.current += 1;
                    }
                    if source[self.current] == b'.' {
                        self.current += 1;
                    }
                    while self.current < source.len() && source[self.current].is_ascii_digit() {
                        self.current += 1;
                    }
                    match std::str::from_utf8(&source[self.start..self.current]) {
                        Ok(number_string) => match number_string.parse() {
                            Ok(number) => self.tokens.push(Token {
                                line: self.line,
                                r#type: TokenType::Number(number),
                            }),
                            Err(_) => self.errors.push(ScanError::InvalidNumber {
                                line: self.line,
                                number: number_string.into(),
                            }),
                        },
                        Err(_) => self.errors.push(ScanError::InvalidUtf8 {
                            line: self.line,
                            bytes: source[self.start..self.current].into(),
                        }),
                    }
                    self.start = self.current;
                }
                _ => {
                    self.errors.push(ScanError::UnexpectedCharacter {
                        character: source[self.current].into(),
                        line: self.line,
                    });
                    self.current += 1;
                    self.start = self.current;
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
        self.current += 1;
        self.start = self.current;
    }
    fn peek(&self, source: &[u8]) -> u8 {
        if self.current + 1 < source.len() {
            source[self.current + 1]
        } else {
            b'\0'
        }
    }
}

fn print_error(line: usize, message: &str) {
    report_error(line, "", message);
}

fn report_error(line: usize, r#where: &str, message: &str) {
    eprintln!("[line {line}] Error {where}: {message}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan() {
        let tokens = scan_tokens(
            b"
                // this is a comment
                (( )){} // grouping stuff
                !*+-/=<> <= == // operators",
        );
        assert_eq!(
            tokens,
            Ok(vec![
                Token {
                    line: 3,
                    r#type: TokenType::LeftBracket,
                },
                Token {
                    line: 3,
                    r#type: TokenType::LeftBracket,
                },
                Token {
                    line: 3,
                    r#type: TokenType::RightBracket,
                },
                Token {
                    line: 3,
                    r#type: TokenType::RightBracket,
                },
                Token {
                    line: 3,
                    r#type: TokenType::LeftBrace,
                },
                Token {
                    line: 3,
                    r#type: TokenType::RightBrace,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Not,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Star,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Plus,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Minus,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Slash,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Assign,
                },
                Token {
                    line: 4,
                    r#type: TokenType::LessThan,
                },
                Token {
                    line: 4,
                    r#type: TokenType::GreaterThan,
                },
                Token {
                    line: 4,
                    r#type: TokenType::LessThanOrEqual,
                },
                Token {
                    line: 4,
                    r#type: TokenType::Equal,
                },
            ])
        )
    }
}
