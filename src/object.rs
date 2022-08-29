use crate::function::Function;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Char(char),
    Boolean(bool),
    Callable(Function),
    None,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Number(value) => write!(f, "{}", value),
            Object::Char(value) => write!(f, "{}", value),
            Object::String(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::None => write!(f, "null"),
            Object::Callable(func) => write!(f, "{:?}", func),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::None, Object::None) => true,
            (_, Object::None) | (Object::None, _) => false,
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            (Object::Number(left), Object::Number(right)) => left == right,
            (Object::String(left), Object::String(right)) => left == right,
            _ => false,
        }
    }
}
