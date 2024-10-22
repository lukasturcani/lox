use crate::scanner::{Token, TokenType};

enum Expr {
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

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Box<Expr> {
        let mut expr = self.unary();
        while self.r#match(&[TokenType::NotEqual, TokenType::Equal]) {
            let operator = self.tokens[self.current - 1].r#type.clone();
            let right = self.unary();
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }
        expr
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.r#match(&[TokenType::NotEqual, TokenType::Equal]) {
            Box::new(Expr::Unary {
                operator: self.tokens[self.current - 1].r#type.clone(),
                right: self.unary(),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Box<Expr> {
        match &self.tokens[self.current].r#type {
            value @ TokenType::Number(_) => Box::new(Expr::Literal {
                value: value.clone(),
            }),
            value @ TokenType::True => Box::new(Expr::Literal {
                value: value.clone(),
            }),
            value @ TokenType::False => Box::new(Expr::Literal {
                value: value.clone(),
            }),
            TokenType::LeftBracket => {
                self.current += 1;
                let expression = self.expression();
                assert_eq!(self.tokens[self.current].r#type, TokenType::RightBracket);
                self.current += 1;
                Box::new(Expr::Grouping { expression })
            }
            _ => panic!("unexpected toknen"),
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
