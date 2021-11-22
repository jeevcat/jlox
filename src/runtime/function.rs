use std::{cell::RefCell, fmt, rc::Rc};

use anyhow::Result;

use super::{environment::Environment, interpreter::Interpreter, value::Value};
use crate::ast::stmt::FunctionDecl;

#[derive(Clone)]
pub struct Function {
    pub declaration: FunctionDecl,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub func: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
    pub name: String,
}

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>;
    fn get_arity(&self) -> u8;
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let mut environment = Environment::with_enclosing(self.closure.clone());

        for (i, argument) in arguments.into_iter().enumerate() {
            environment.define(&self.declaration.params[i].lexeme, Some(argument));
        }

        let old_return_value = interpreter.return_value.clone();
        interpreter.execute_block(&self.declaration.body, environment)?;
        let return_value = interpreter.return_value.clone();
        interpreter.return_value = old_return_value;
        Ok(return_value.unwrap_or(Value::Nil))
    }

    fn get_arity(&self) -> u8 {
        self.declaration.params.len().try_into().unwrap()
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

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("<fun {}>", self.declaration.name.lexeme))
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("<fun {}>", self.name))
    }
}
