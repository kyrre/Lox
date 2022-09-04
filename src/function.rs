use crate::environment::Environment;
use crate::errors::{Result, Error};
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::statement::Stmt;
use crate::tokens::Token;
use std::rc::Rc;
use std::cell::RefCell;

// We either need to split up the Object
// and Object type
// OR 
// implement some custom traits

#[derive(Clone, Debug)]
pub enum Function {
    User {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>
    },
    Native {
        body: Box<fn(Vec<Object>) -> Object>,
        arity: usize,
    },
}

impl Function {
    pub fn call(&self, 
        interpreter: &mut Interpreter, 
        arguments: Vec<Object>) -> Result<Object> {
        let res = match self {
            Function::Native { body, ..}  => Ok(body(arguments)),
            Function::User { params, body, closure , ..} => {

                let environment = Rc::new(RefCell::new(Environment::new(&closure)));
                for (param, arg) in params.iter().zip(arguments.iter()){
                    environment.borrow_mut().define(param.lexeme.clone(), arg.clone());
                }

                match interpreter.execute_block(body, environment) {
                    Err(Error::Return{value}) => Ok(value),
                    Err(other) => Err(other),
                    Ok(..) => Ok(Object::None)
                }
            }
        };

        println!("call - result = {:?}", res);

        res
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, ..} => *arity,
            Function::User { params, ..} => params.len(),
        }
    }
}
