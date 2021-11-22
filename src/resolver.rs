use std::collections::HashMap;

use crate::{
    ast::{
        expr::Expr,
        stmt::{FunctionDecl, Stmt},
    },
    error::report_error,
    runtime::interpreter::Interpreter,
    scanner::Token,
};

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_statements(statements);
                self.end_scope();
            }
            Stmt::Expression(expression) => {
                self.resolve_expression(expression);
            }
            Stmt::FunctionDecl(decl) => {
                self.declare(&decl.name);
                self.define(&decl.name);
                self.resolve_function(decl, FunctionType::Function);
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
            }
            Stmt::Print(expression) => {
                self.resolve_expression(expression);
            }
            Stmt::Return { keyword, value } => {
                if let Some(expression) = value {
                    if self.current_function == FunctionType::None {
                        report_error(keyword, "Can't return from top-level code");
                    }
                    self.resolve_expression(expression);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
            Stmt::VarDecl { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer);
                }
                self.define(name);
            }
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Assign { name, value } => {
                self.resolve_expression(value);
                self.resolve_local(expression, name);
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call { callee, arguments } => {
                self.resolve_expression(callee);

                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
            Expr::Grouping(expression) => {
                self.resolve_expression(expression);
            }
            Expr::Literal(_) => {}
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Unary { operator: _, right } => {
                self.resolve_expression(right);
            }
            Expr::Variable { name } => {
                if let Some(top) = self.scopes.last() {
                    if let Some(is_defined) = top.get(&name.lexeme) {
                        if is_defined == &false {
                            report_error(name, "Can't read local variable in its own initializer");
                        }
                    }
                }
                self.resolve_local(expression, name);
            }
        }
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expression, i.try_into().unwrap());
                return;
            }
        }
    }

    fn resolve_function(&mut self, decl: &FunctionDecl, func_type: FunctionType) {
        let enclosing_function = std::mem::replace(&mut self.current_function, func_type);
        self.begin_scope();
        for name in &decl.params {
            self.declare(name);
            self.define(name);
        }
        self.resolve_statements(&decl.body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(top) = self.scopes.last_mut() {
            if top.contains_key(&name.lexeme) {
                report_error(name, "Already a variable with this name in this scope");
            }
            top.insert(name.lexeme.to_string(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.lexeme.to_string(), true);
        }
    }
}
