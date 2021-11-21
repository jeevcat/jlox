use super::expr::Expr;

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
        name: String,
        initializer: Option<Expr>,
    },
}

#[derive(Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}
