use crate::errors::{Error, Result};
use crate::tokens::{Literal, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::default(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal> {
        let _name = &name.lexeme;
        if self.values.contains_key(_name) {
            self.values.insert(_name.clone(), value.clone());
            Ok(value)
        } else {
            self.enclosing.as_mut().map_or_else(
                || Err(Error::Runtime(format!("Undefined variable {}", name))),
                |enclosing| enclosing.borrow_mut().assign(name, value),
            )
        }
    }

    // some &string template magic here?
    pub fn get(&self, name: &Token) -> Result<Literal> {
        let value = self.values.get(&name.lexeme).cloned();

        match (value, &self.enclosing) {
            (Some(v), _) => Ok(v),
            (None, Some(enclosing)) => enclosing.borrow().get(name),
            (None, None) => Err(Error::Runtime(format!(
                "Undefined variable '{}'.",
                &name.lexeme
            ))),
        }
    }
}
