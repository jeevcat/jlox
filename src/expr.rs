use crate::scanner::Token;
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
