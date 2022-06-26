use std::{fmt, error};

#[derive(Debug, Clone)]
pub enum LoxError {
    Parse, 
    Runtime,
    Scanner,
}

impl error::Error for LoxError {}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lox generic error!")
    }
}

pub type Result<T> = std::result::Result<T, LoxError>;
