use crate::errors::{Error, Result};
use crate::tokens::{Literal, Token};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal> {
        let name = &name.lexeme;
        if self.values.contains_key(name) {
            self.values.insert(name.clone(), value.clone());
            Ok(value)
        } else {
            Err(Error::Runtime(format!("Undefined variable {}", name)))
        }
    }

    // some &string template magic here?
    pub fn get(&self, name: &Token) -> Result<Literal> {
        self.values
            .get(&name.lexeme)
            .cloned()
            .ok_or(Error::Runtime(format!(
                "Undefined variable '{}'.",
                &name.lexeme
            )))
    }
}
