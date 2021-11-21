use anyhow::Result;

use super::{environment::Environment, interpreter::Interpreter, value::Value};
use crate::ast::stmt::FunctionDecl;

#[derive(Clone)]
pub struct Function {
    pub declaration: FunctionDecl,
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub func: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
}

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>;
    fn get_arity(&self) -> u8;
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let mut environment = Environment::with_enclosing(interpreter.globals.clone());

        for (i, argument) in arguments.into_iter().enumerate() {
            environment.define(&self.declaration.params[i], Some(argument));
        }

        interpreter.execute_block(&self.declaration.body, environment)?;
        Ok(Value::Nil)
    }

    fn get_arity(&self) -> u8 {
        self.declaration.params.len() as u8
    }
}

impl Callable for NativeFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        (self.func)(interpreter, arguments)
    }

    fn get_arity(&self) -> u8 {
        self.arity
    }
}
