use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};

use super::{environment::Environment, function::Function, value::Value};
use crate::{
    ast::{
        expr::{Expr, Literal},
        stmt::Stmt,
    },
    runtime::function::{Callable, NativeFunction},
    scanner::TokenType,
};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    // Used to unwind call stack when nested return is called
    pub return_value: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define(
            "clock",
            Some(Value::NativeFunction(NativeFunction {
                arity: 0,
                func: |_, _| {
                    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    Ok(Value::Number(since_the_epoch.as_secs_f64()))
                },
            })),
        );
        let globals = Rc::new(RefCell::new(globals));
        Self {
            environment: globals.clone(),
            globals,
            return_value: None,
        }
    }

    pub fn execute(&mut self, statement: &Stmt) -> Result<()> {
        if self.return_value.is_some()
        {
            // Unwind stack
            return Ok(())
        }

        match statement {
            Stmt::Block(statements) => self.execute_block(
                statements,
                Environment::with_enclosing(self.environment.clone()),
            ),
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                // Discard result of interpret
                Ok(())
            }
            Stmt::FunctionDecl(declaration) => {
                let function = Function {
                    declaration: declaration.clone(),
                };
                self.environment
                    .borrow_mut()
                    .define(&declaration.name, Some(Value::Function(function)));
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(expr)?;
                println!("{}", val);
                Ok(())
            }
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(expr) => self.evaluate(expr)?,
                    _ => Value::Nil,
                };
                self.return_value = Some(value);
                Ok(())
            }
            Stmt::VarDecl { name, initializer } => {
                let value = if let Some(i) = initializer {
                    Some(self.evaluate(i)?)
                } else {
                    None
                };
                self.environment.borrow_mut().define(name, value);
                Ok(())
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(())
            }
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt], environment: Environment) -> Result<()> {
        let prev = self.environment.clone();
        let execute_statements = || -> Result<()> {
            self.environment = Rc::new(RefCell::new(environment));

            for statement in statements {
                self.execute(statement)?;
            }
            Ok(())
        };
        let result = execute_statements();
        self.environment = prev;

        result
    }

    pub fn evaluate(&mut self, expression: &Expr) -> Result<Value> {
        match expression {
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                Ok(self.environment.borrow_mut().assign(name, value)?)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator {
                    TokenType::Minus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        _ => Err(anyhow!("Operands must be two numbers or two strings.")),
                    },
                    TokenType::Greater => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left > right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left >= right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::Less => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left < right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left <= right))
                        }
                        _ => Err(error_number()),
                    },
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
                    _ => unreachable!(),
                }
            }
            Expr::Call { callee, arguments } => {
                let callee = self.evaluate(callee)?;
                let mut result = vec![];
                for argument in arguments {
                    result.push(self.evaluate(argument)?);
                }

                let arity = match &callee {
                    Value::NativeFunction(f) => f.get_arity(),
                    Value::Function(f) => f.get_arity(),
                    _ => return Err(anyhow!("Can only call functions and classes")),
                };

                if (arguments.len() as u8) != arity {
                    return Err(anyhow!(
                        "Expected {} arguments but got {}",
                        arity,
                        arguments.len()
                    ));
                }

                match callee {
                    Value::NativeFunction(f) => f.call(self, result),
                    Value::Function(f) => f.call(self, result),
                    _ => unreachable!(),
                }
            }
            Expr::Grouping(g) => self.evaluate(g),
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.to_string()),
                Literal::True => Value::Boolean(true),
                Literal::False => Value::Boolean(false),
                Literal::Nil => Value::Nil,
            }),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                match operator {
                    TokenType::Or => {
                        if left.is_truthy() {
                            return Ok(left);
                        } else {
                        }
                    }
                    TokenType::And => {
                        if !left.is_truthy() {
                            return Ok(left);
                        } else {
                        }
                    }
                    _ => unreachable!(),
                }
                self.evaluate(right)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(error_number()),
                    },
                    TokenType::Bang => Ok(Value::Boolean(!right.is_truthy())),
                    _ => unreachable!(),
                }
            }
            Expr::Variable { name } => Ok(self.environment.borrow().get(name)?),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn error_number() -> anyhow::Error {
    anyhow!("Operand must be a number.")
}
