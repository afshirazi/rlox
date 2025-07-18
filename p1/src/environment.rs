use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expr::Literal;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>, // reference to parent environment, for scoping
    map: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            map: HashMap::new(),
        }
    }

    pub fn with_enclosing(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(parent),
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Literal> {
        self.map
            .get(key)
            .map(|lit| lit.clone())
            .or_else(|| self.enclosing.as_ref()?.borrow().get(key))
    }

    pub fn define(&mut self, key: String, val: Literal) {
        self.map.insert(key, val);
    }

    pub fn assign(&mut self, key: String, val: Literal) -> Result<(), String> {
        if self.map.contains_key(&key) {
            self.map.insert(key, val);
            Ok(())
        } else if let Some(env) = self.enclosing.as_ref() {
            env.borrow_mut().assign(key, val)
        } else {
            Err(format!("Undefined variable: {}", key))
        }
    }
}
