use std::{collections::HashMap, rc::Rc};

use crate::expr::Literal;

pub struct Environment {
    enclosing: Option<Rc<Environment>>, // reference to parent environment, for scoping
    map: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            map: HashMap::new(),
        }
    }

    pub fn with_enclosing(parent: Rc<Environment>) -> Self {
        Self {
            enclosing: Some(parent),
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Literal> {
        if let Some(lit) = self.map.get(key) {
            Some(lit)
        } else {
            match &self.enclosing {
                Some(env) => env.get(key),
                None => None,
            }
        }
    }

    pub fn define(&mut self, key: String, val: Literal) {
        self.map.insert(key, val);
    }

    pub fn assign(&mut self, key: String, val: Literal) -> Result<(), String> {
        if self.map.contains_key(&key) {
            self.map.insert(key, val);
            Ok(())
        } else {
            Err(format!("Undefined variable: {}", key))
        }
    }
}
