use super::expr::Expr;
use crate::scanner::Token;

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    FunctionDecl(FunctionDecl),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return(Option<Expr>),
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    VarDecl {
        name: Token,
        initializer: Option<Expr>,
    },
}

#[derive(Clone)]
pub struct FunctionDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}
