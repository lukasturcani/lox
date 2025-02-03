use crate::scanner::{Token, TokenType};

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: TokenType,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: TokenType,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: TokenType,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: TokenType,
    },
    Logical {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: TokenType,
        value: Box<Expr>,
    },
    Super {
        keyword: TokenType,
        method: TokenType,
    },
    This {
        keyword: TokenType,
    },
    Unary {
        operator: TokenType,
        right: Box<Expr>,
    },
    Variable {
        name: TokenType,
    },
}

pub enum Statement {}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();
    let mut parser = Parser::new(tokens);
    while parser.current < parser.tokens.len() {
        statements.push(parser.statement()?);
    }
    Ok(statements)
}

#[derive(Clone, Copy, Debug)]
pub struct ParseError;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn synchronize(&mut self) {
        self.current += 1;
        while self.current < self.tokens.len() {
            if self.tokens[self.current - 1].r#type == TokenType::Semicolon {
                return;
            }
            if let Some(token) = self.tokens.get(self.current + 1) {
                match token.r#type {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => {}
                }
            }

            self.current += 1;
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        todo!()
    }

    fn expression(&mut self) -> Result<Box<Expr>, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, ParseError> {
        let mut expr = self.comparison()?;
        while self.r#match(&[TokenType::NotEqual, TokenType::Equal]) {
            let operator = self.tokens[self.current - 1].r#type.clone();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>, ParseError> {
        let mut expr = self.term()?;
        while self.r#match(&[
            TokenType::GreaterThan,
            TokenType::GreaterThanOrEqual,
            TokenType::LessThan,
            TokenType::LessThanOrEqual,
        ]) {
            let operator = self.tokens[self.current - 1].r#type.clone();
            let right = self.term()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, ParseError> {
        let mut expr = self.factor()?;
        while self.r#match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.tokens[self.current - 1].r#type.clone();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>, ParseError> {
        let mut expr = self.unary()?;
        while self.r#match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.tokens[self.current - 1].r#type.clone();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>, ParseError> {
        if self.r#match(&[TokenType::NotEqual, TokenType::Equal]) {
            Ok(Box::new(Expr::Unary {
                operator: self.tokens[self.current - 1].r#type.clone(),
                right: self.unary()?,
            }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, ParseError> {
        match &self.tokens[self.current].r#type {
            value @ TokenType::Number(_) => {
                self.current += 1;
                Ok(Box::new(Expr::Literal {
                    value: value.clone(),
                }))
            }
            value @ TokenType::String(_) => {
                self.current += 1;
                Ok(Box::new(Expr::Literal {
                    value: value.clone(),
                }))
            }
            value @ TokenType::True => {
                self.current += 1;
                Ok(Box::new(Expr::Literal {
                    value: value.clone(),
                }))
            }
            value @ TokenType::False => {
                self.current += 1;
                Ok(Box::new(Expr::Literal {
                    value: value.clone(),
                }))
            }
            value @ TokenType::Nil => {
                self.current += 1;
                Ok(Box::new(Expr::Literal {
                    value: value.clone(),
                }))
            }
            TokenType::LeftBracket => {
                self.current += 1;
                let expression = self.expression()?;
                assert_eq!(self.tokens[self.current].r#type, TokenType::RightBracket);
                self.current += 1;
                Ok(Box::new(Expr::Grouping { expression }))
            }
            _ => Err(ParseError),
        }
    }

    fn r#match(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if std::mem::discriminant(&self.tokens[self.current].r#type)
                == std::mem::discriminant(token)
            {
                self.current += 1;
                return true;
            }
        }
        false
    }
}
