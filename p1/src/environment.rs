use std::collections::HashMap;

use crate::expr::Literal;

pub struct Environment {
    enclosing: Option<Box<Environment>>, // reference to parent environment, for scoping
    map: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            map: HashMap::new(),
        }
    }

    pub fn with_enclosing(parent: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(parent)),
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Literal> {
        self.map.get(key)
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
