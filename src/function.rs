use crate::errors::Result;
use crate::tokens::Literal;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Function {
    User,
    Native {
        body: Box<fn(Vec<Literal>) -> Literal>,
        arity: usize,
    },
}

impl Function {
    pub fn call(&self, arguments: Vec<Literal>) -> Result<Literal> {
        Ok(Literal::Boolean(true))
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { body, arity } => *arity,
            _ => 10,
        }
    }
}
