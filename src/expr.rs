use crate::scanner::{Token, TokenType};
use anyhow::{anyhow, Result};
use std::fmt::{self, Debug, Write};

pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping(Box<Expr<'a>>),
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Literal(Literal<'a>),
}

impl<'a> fmt::Debug for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, operator.lexeme, &[left, right]),
            Self::Grouping(expression) => parenthesize(f, "group", &[expression]),
            Self::Unary { operator, right } => parenthesize(f, operator.lexeme, &[right]),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}

fn parenthesize(f: &mut fmt::Formatter, name: &str, exprs: &[&Expr]) -> fmt::Result {
    f.write_char('(')?;
    f.write_str(name)?;
    for expr in exprs.iter() {
        f.write_char(' ')?;
        expr.fmt(f)?;
    }
    f.write_char(')')?;
    Ok(())
}

pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => std::fmt::Display::fmt(&b, f),
            Value::Number(n) => std::fmt::Display::fmt(&n, f),
            Value::String(s) => f.write_str(s),
        }
    }
}

fn error_number() -> anyhow::Error {
    anyhow!("Operand must be a number.")
}

impl<'a> Expr<'a> {
    pub fn evaluate(&self) -> Result<Value> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate()?;
                let right = right.evaluate()?;

                match operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        _ => Err(anyhow!("Operands must be two numbers or two strings.")),
                    },
                    TokenType::Greater => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left > right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left >= right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Less => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left < right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left <= right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
                    _ => unreachable!(),
                }
            }
            Expr::Grouping(g) => g.evaluate(),
            Expr::Unary { operator, right } => {
                let right = right.evaluate()?;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(error_number()),
                    },
                    TokenType::Bang => Ok(Value::Boolean(!right.is_truthy())),
                    _ => unreachable!(),
                }
            }
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.to_string()),
                Literal::True => Value::Boolean(true),
                Literal::False => Value::Boolean(false),
                Literal::Nil => Value::Nil,
            }),
        }
    }
}

pub enum Literal<'a> {
    Number(f64),
    String(&'a str),
    True,
    False,
    Nil,
}

impl<'a> fmt::Debug for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => num.fmt(f),
            Self::String(str) => str.fmt(f),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr::{Expr, Literal},
        scanner::{Token, TokenType},
    };

    #[test]
    fn debug_print() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1.))),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+",
                line: 1,
                col: 0,
            },
            right: Box::new(Expr::Literal(Literal::Number(2.))),
        };
        assert_eq!(format!("{:?}", expr), "(+ 1.0 2.0)")
    }

    #[test]
    fn debug_print2() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-",
                    line: 1,
                    col: 0,
                },
                right: Box::new(Expr::Literal(Literal::Number(123.))),
            }),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*",
                line: 1,
                col: 0,
            },
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        };
        assert_eq!(format!("{:?}", expr), "(* (- 123.0) (group 45.67))")
    }
}
