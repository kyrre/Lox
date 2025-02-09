use std::{error, fmt};
use crate::object::Object;

#[derive(Debug, Clone)]
pub enum Error {
    Parse,
    Runtime(String),
    Syntax,
    Scanner,
    Return{value: Object}
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Runtime(s) => write!(f, "{}", s),
            _ => write!(f, "Lox generic error!"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
