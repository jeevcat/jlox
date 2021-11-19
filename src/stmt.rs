use crate::{expr::Expr, scanner::Token};
use anyhow::Result;

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Var {
        name: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
}

impl<'a> Stmt<'a> {
    pub fn execute(&self) -> Result<()> {
        match self {
            Stmt::Expression(expr) => {
                expr.evaluate()?;
                // Discard result of interpret
                Ok(())
            }
            Stmt::Print(expr) => {
                let val = expr.evaluate()?;
                println!("{}", val);
                Ok(())
            }
            Stmt::Var { name, initializer } => todo!(),
        }
    }
}
