use crate::errors::{Error, Result};
use crate::interpreter::{self, Interpreter};
use crate::object::Object;
use crate::tokens::Token;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Class { name }
    }

    pub fn call(self, interpreter: &Interpreter, arguments: &Vec<Object>) -> Instance {
        Instance::new(self)
    }

    pub fn arity() -> usize {
        0
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    class: Class, // should this be Rc<RefCell ??
    fields: HashMap<String, Object>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Instance {
            class,
            fields: HashMap::default(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        match self.fields.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => Err(Error::Runtime(format!(
                "Undefined property '{}'.",
                name.lexeme
            ))),
        }
    }
}
