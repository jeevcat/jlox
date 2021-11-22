use std::fmt;

use crate::scanner::{Number, Token, TokenType};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    Unary {
        operator: TokenType,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Literal {
    Number(Number),
    String(String),
    True,
    False,
    Nil,
}

impl fmt::Debug for Literal {
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
