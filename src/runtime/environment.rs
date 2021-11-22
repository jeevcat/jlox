use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use anyhow::{anyhow, Result};

use super::value::Value;

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

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), Some(value.clone()));
            return Ok(value);
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.deref().borrow_mut().assign(name, value);
        }

        Err(anyhow!("Undefined variable '{}'", name))
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(got) = self.get_internal(name) {
            return Ok(got.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.deref().borrow().get(name);
        }
        Err(anyhow!("Undefined variable '{}'", name))
    }

    pub fn get_at(&self, distance: u32, name: &str) -> Result<Value> {
        if distance == 0 {
            return self.get(name);
        }
        // The unwrap here is safe if we trust our resolver
        self.enclosing
            .as_deref()
            .unwrap()
            .borrow()
            .get_at(distance - 1, name)
    }

    pub fn assign_at(&mut self, distance: u32, name: &str, value: Value) -> Result<Value> {
        if distance == 0 {
            return self.assign(name, value);
        }
        // The unwrap here is safe if we trust our resolver
        self.enclosing
            .as_deref()
            .unwrap()
            .borrow_mut()
            .assign_at(distance - 1, name, value)
    }

    fn get_internal(&self, name: &str) -> Option<&Value> {
        self.values.get(name)?.as_ref()
    }
}
