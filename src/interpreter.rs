use crate::{
    environment::Environment,
    expr::{Expr, Literal, Value},
    scanner::TokenType,
    stmt::Stmt,
};
use anyhow::{anyhow, Result};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn execute(&mut self, statement: &Stmt) -> Result<()> {
        match statement {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                // Discard result of interpret
                Ok(())
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(expr)?;
                println!("{}", val);
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(i) = initializer {
                    Some(self.evaluate(i)?)
                } else {
                    None
                };
                self.env.define(name.lexeme, value);
                Ok(())
            }
        }
    }

    pub fn evaluate(&self, expression: &Expr) -> Result<Value> {
        match expression {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.token_type {
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
            Expr::Grouping(g) => self.evaluate(g),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(error_number()),
                    },
                    TokenType::Bang => Ok(Value::Boolean(!right.is_truthy())),
                    _ => unreachable!(),
                }
            }
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(n) => Value::Number(*n),
                Literal::String(s) => Value::String(s.to_string()),
                Literal::True => Value::Boolean(true),
                Literal::False => Value::Boolean(false),
                Literal::Nil => Value::Nil,
            }),
            Expr::Variable { name } => self.env.get(name),
        }
    }
}

fn error_number() -> anyhow::Error {
    anyhow!("Operand must be a number.")
}
