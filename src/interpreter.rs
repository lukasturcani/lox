use crate::{parser::Expr, scanner::TokenType};

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
    fn truthy(&self) -> Value {
        match self {
            b @ Self::Bool(_) => b.clone(),
            Self::Nil => Self::Bool(false),
            _ => Self::Bool(true),
        }
    }

    fn equals(&self, other: &Self) -> Value {
        match (self, other) {
            (Self::Nil, Self::Nil) => Self::Bool(true),
            (Self::Bool(a), Self::Bool(b)) => Self::Bool(a == b),
            (Self::Number(a), Self::Number(b)) => Self::Bool(a == b),
            (Self::String(a), Self::String(b)) => Self::Bool(a == b),
            _ => Self::Bool(false),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    Type,
}

pub fn interpret(expr: &Box<Expr>) -> Result<Value, Error> {
    match &**expr {
        Expr::Literal { value } => match value {
            TokenType::Number(value) => Ok(Value::Number(*value)),
            TokenType::String(value) => Ok(Value::String(value.clone())),
            TokenType::True => Ok(Value::Bool(true)),
            TokenType::False => Ok(Value::Bool(false)),
            TokenType::Nil => Ok(Value::Nil),
            _ => unreachable!(),
        },
        Expr::Grouping { expression } => interpret(expression),
        Expr::Unary { operator, right } => {
            let right = interpret(right)?;
            match operator {
                TokenType::Minus => {
                    if let Value::Number(value) = right {
                        Ok(Value::Number(-value))
                    } else {
                        Err(Error::Type)
                    }
                }
                TokenType::Bang => {
                    if let Value::Bool(value) = right.truthy() {
                        Ok(Value::Bool(!value))
                    } else {
                        Err(Error::Type)
                    }
                }
                _ => unreachable!("{operator:?}"),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = interpret(left)?;
            let right = interpret(right)?;
            match operator {
                TokenType::Minus => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Number(left - right))
                }
                TokenType::Slash => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Number(left / right))
                }
                TokenType::Star => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Number(left * right))
                }
                TokenType::Plus => {
                    if let (Value::Number(left), Value::Number(right)) = (&left, &right) {
                        Ok(Value::Number(left + right))
                    } else if let (Value::String(left), Value::String(right)) = (&left, &right) {
                        Ok(Value::String(left.clone() + right))
                    } else {
                        Err(Error::Type)
                    }
                }
                TokenType::GreaterThan => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Bool(left > right))
                }
                TokenType::GreaterThanOrEqual => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Bool(left >= right))
                }
                TokenType::LessThan => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Bool(left < right))
                }
                TokenType::LessThanOrEqual => {
                    let Value::Number(left) = left else {
                        return Err(Error::Type);
                    };
                    let Value::Number(right) = right else {
                        return Err(Error::Type);
                    };
                    Ok(Value::Bool(left <= right))
                }
                TokenType::Equal => Ok(left.equals(&right)),
                TokenType::NotEqual => {
                    let Value::Bool(r) = left.equals(&right) else {
                        unreachable!()
                    };
                    Ok(Value::Bool(!r))
                }
                other => {
                    unreachable!("{:?}", other)
                }
            }
        }
        _ => unreachable!(),
    }
}
