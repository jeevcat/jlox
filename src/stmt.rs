use crate::expr::Expr;
use anyhow::Result;

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
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
        }
    }
}
