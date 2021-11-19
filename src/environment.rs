use anyhow::{anyhow, Result};
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use crate::{expr::Value, scanner::Token};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_nested(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value> {
        if self.values.contains_key(name.lexeme) {
            self.values
                .insert(name.lexeme.to_owned(), Some(value.clone()));
            return Ok(value);
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.deref().borrow_mut().assign(name, value);
        }

        Err(anyhow!("Undefined variable '{}'", name.lexeme))
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        if let Some(got) = self.get_internal(name.lexeme) {
            return Ok(got.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.deref().borrow().get(name);
        }
        Err(anyhow!("Undefined variable '{}'", name.lexeme))
    }

    fn get_internal(&self, name: &str) -> Option<&Value> {
        self.values.get(name)?.as_ref()
    }
}
