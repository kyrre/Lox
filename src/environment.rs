use crate::errors::{Error, Result};
use crate::object::Object;
use crate::tokens::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::default(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        // println!("defining: {:?}", name);
        self.values.insert(name, value);
        // println!("{:?}", self.values);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<Object> {
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

    pub fn get(&self, name: &Token) -> Result<Object> {
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

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object> {

        //println!("distance = {}", distance);
        //println!("self.values = {:?}", self.values);
        //println!("name = {:?}", name);
        // why is distance set to 1 ???


        // println!("{}", distance);
        // println!("self.values = {:?}", self.values);
        let value;
        if distance > 0 {
            value = self
                .ancestor(distance)
                .borrow()
                .values
                .get(&name.lexeme)
                .cloned();
        } else {
            value = self.values.get(&name.lexeme).cloned();
        }

        // println!("value = {:?}", value);

        value.ok_or(Error::Runtime(format!(
            "get_at - Undefined variable {}",
            name.lexeme
        )))
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Object) -> Result<Object> {
        let name = &name.lexeme;
        let v = value.clone();

        if distance > 0 {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.clone(), value);
        } else {
            self.values.insert(name.clone(), value);
        }

        Ok(v)
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        // how to hande self.enclosing = None ?
        let mut environment = Rc::clone(&self.enclosing.clone().unwrap());

        // traverese
        for _ in 1..distance {
            let parent = environment.borrow().enclosing.clone().unwrap();
            environment = Rc::clone(&parent);
        }

        // println!("environment = {:?}", environment);

        environment
    }
}
