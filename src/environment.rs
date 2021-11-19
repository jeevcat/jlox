use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::{expr::Value, scanner::Token};

pub struct Environment {
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        self.get_internal(name.lexeme)
            .cloned()
            .ok_or_else(|| anyhow!("Undefined variable '{}'", name.lexeme))
    }

    fn get_internal(&self, name: &str) -> Option<&Value> {
        self.values.get(name)?.as_ref()
    }
}
