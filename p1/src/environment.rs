use std::collections::HashMap;

use crate::expr::Literal;

pub struct Environment {
    map: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Literal> {
        self.map.get(key)
    }

    pub fn define(&mut self, key: String, val: Literal) {
        self.map.insert(key, val);
    }
}
