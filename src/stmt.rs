use crate::{expr::Expr, scanner::Token};

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Var {
        name: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    Block(Vec<Stmt<'a>>),
}
