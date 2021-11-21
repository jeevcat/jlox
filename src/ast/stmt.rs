use super::expr::Expr;

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    FunctionDecl(FunctionDecl),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    VarDecl {
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
}

#[derive(Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}
