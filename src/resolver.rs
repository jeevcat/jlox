use std::collections::HashMap;

use anyhow::Result;

use crate::{
    ast::{
        expr::Expr,
        stmt::{FunctionDecl, Stmt},
    },
    error::error,
    runtime::interpreter::Interpreter,
    scanner::Token,
};

struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    fn resolve_statement(&mut self, statement: &Stmt) -> Result<()> {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_statements(statements)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Expression(expression) => {
                self.resolve_expression(expression);
                Ok(())
            }
            Stmt::FunctionDecl(decl) => {
                self.declare(&decl.name);
                self.define(&decl.name);
                self.resolve_function(decl);
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
                Ok(())
            }
            Stmt::Print(expression) => {
                self.resolve_expression(expression);
                Ok(())
            }
            Stmt::Return(expression) => {
                if let Some(expression) = expression {
                    self.resolve_expression(expression);
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
                Ok(())
            }
            Stmt::VarDecl { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer)?;
                }
                self.define(name);
                Ok(())
            }
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) -> Result<()> {
        match expression {
            Expr::Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(expression, name);
                Ok(())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
                Ok(())
            }
            Expr::Call { callee, arguments } => {
                self.resolve_expression(callee);

                for argument in arguments {
                    self.resolve_expression(argument);
                }

                Ok(())
            }
            Expr::Grouping(expression) => {
                self.resolve_expression(expression);
                Ok(())
            }
            Expr::Literal(_) => Ok(()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
                Ok(())
            }
            Expr::Unary { operator, right } => {
                self.resolve_expression(right);
                Ok(())
            }
            Expr::Variable { name } => {
                if let Some(top) = self.scopes.last() {
                    if let Some(is_defined) = top.get(&name.lexeme) {
                        if is_defined == &false {
                            return Err(error(
                                name,
                                "Can't read local variable in its own initializer",
                            ));
                        }
                    }
                }
                self.resolve_local(expression, name);
                Ok(())
            }
        }
    }

    fn resolve_statements(&mut self, statements: &[Stmt]) -> Result<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expression, i.try_into().unwrap());
                return;
            }
        }
    }

    fn resolve_function(&mut self, decl: &FunctionDecl) {
        self.begin_scope();
        for name in &decl.params {
            self.declare(name);
            self.define(name);
        }
        self.resolve_statements(&decl.body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.lexeme.to_string(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.lexeme.to_string(), true);
        }
    }
}
