use crate::{expr::Expr, scanner::Token};

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    Print(Expr<'a>),
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
    Var {
        name: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    Block(Vec<Stmt<'a>>),
}
