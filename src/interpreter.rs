#![allow(dead_code, unused, non_camel_case_types, non_snake_case)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::default;
use std::f32::MIN;

use crate::ast::{Expr, Visitor as ExprVisitor};
use crate::class::Class;
use crate::environment::{self, Environment};
use crate::errors::{Error, Result};
use crate::function::Function;
use crate::object::Object;
use crate::statement::{self, Stmt, Visitor as StmtVisitor};
use crate::tokens::{
    Literal, Token,
    TokenType::{self, *},
};

use std::rc::Rc;

use Object::{Boolean, None as Null, Number, String};

fn clock_fun(args: Vec<Object>) -> Object {
    Object::Number(10.0)
}

#[derive(Default, Debug)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals: Rc<RefCell<Environment>> = Default::default();
        let environment = Rc::clone(&globals);

        let clock = Object::Callable(Function::Native {
            body: Box::new(clock_fun),
            arity: 0,
        });

        globals.borrow_mut().define("clock".to_string(), clock);

        Interpreter {
            globals,
            environment,
            locals: HashMap::default(),
        }
    }
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        statements.iter().try_for_each(|statement| {
            // println!("Executing {:?}", statement);
            self.execute(statement)
        })
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        expr.accept(self)
    }

    pub fn is_truthy(&self, literal: &Object) -> bool {
        match *literal {
            Null => false,
            Boolean(false) => false,
            _ => true,
        }
    }

    pub fn execute(&mut self, statement: &Stmt) -> Result<()> {
        statement.accept(self)
    }

    pub fn resolve(&mut self, name: &Token, i: usize) {
        // println!("resolving {:?} with distance {}", name, i);
        self.locals.insert(name.clone(), i);
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        let result = statements.iter().try_for_each(|statement| {
            let res = self.execute(statement);
            res
        });
        self.environment = previous;
        result
    }

    fn lookup_variable(&self, name: &Token) -> Result<Object> {
        // println!("locals = {:?}", self.locals);
        if let Some(distance) = self.locals.get(name) {
            // println!("{:?}", self.environment.borrow());
            self.environment.borrow().get_at(*distance, name)
        } else {
            // println!("looking in globals!!");
            self.globals.borrow().get(name)
        }
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Object> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (&operator.token_type, &left, &right) {
            (GREATER, Number(left), Number(right)) => Ok(Boolean(left > right)),
            (GREATER_EQUAL, Number(left), Number(right)) => Ok(Boolean(left >= right)),
            (LESS, Number(left), Number(right)) => Ok(Boolean(left < right)),
            (LESS_EQUAL, Number(left), Number(right)) => Ok(Boolean(left <= right)),

            (BANG_EQUAL, left, right) => Ok(Boolean(left != right)),
            (EQUAL_EQUAL, left, right) => Ok(Boolean(left == right)),

            (MINUS, Number(left), Number(right)) => Ok(Number(left - right)),
            (SLASH, Number(left), Number(right)) => Ok(Number(left / right)),
            (STAR, Number(left), Number(right)) => Ok(Number(left * right)),
            (PLUS, Number(left), Number(right)) => Ok(Number(left + right)),
            (PLUS, String(left), String(right)) => Ok(String(left.clone() + right)),

            _ => {
                // println!("off the rails!");
                Err(Error::Runtime(format!(
                    "Error evaluating {} {} {}",
                    left, operator, right
                )))
            }
        }
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Object> {
        let right = self.evaluate(right)?;

        match (&operator.token_type, &right) {
            (MINUS, Number(val)) => Ok(Number(-val)),
            (MINUS, _) => Err(Error::Runtime(format!(
                "Tried to negate invalid operand {:?}",
                right
            ))),
            (BANG, _) => Ok(Boolean(self.is_truthy(&right))),
            _ => Err(Error::Runtime(format!(
                "Invalid unary expr: {:?} {:?}",
                operator, right,
            ))),
        }
    }
    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<Object> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&mut self, value: &Literal) -> Result<Object> {
        match value {
            Literal::Boolean(b) => Ok(Object::Boolean(*b)),
            Literal::None => Ok(Object::None),
            Literal::Number(n) => Ok(Object::Number(*n)),
            Literal::String(s) => Ok(Object::String(s.clone())),
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<Object> {
        self.lookup_variable(name)
    }

    fn visit_variable_assignment_expr(&mut self, expr: &Expr) -> Result<Object> {
        if let Expr::Assign { name, value: expr } = expr {
            let value = self.evaluate(expr)?;
            if let Some(distance) = self.locals.get(name) {
                self.environment
                    .borrow_mut()
                    .assign_at(distance.clone(), name, value)
            } else {
                self.globals.borrow_mut().assign(name, value)
            }
        } else {
            Err(Error::Runtime(format!("Something is very wrong!")))
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Result<Object> {
        if let Expr::Call {
            callee,
            paren,
            arguments,
        } = expr
        {
            let callee = self.evaluate(callee)?;
            let arguments = arguments
                .iter()
                .map(|x| self.evaluate(x))
                .collect::<Result<Vec<Object>>>()?;

            if let Object::Callable(func) = callee {
                if arguments.len() != func.arity() {
                    Err(Error::Runtime(format!(
                        "{:?} Expected {} arguments but got {}.",
                        paren,
                        func.arity(),
                        arguments.len()
                    )))
                } else {
                    func.call(self, arguments)
                }
            } else if let Object::Class(class) = callee {
                let instance = class.call(self, &arguments);
                Ok(Object::Instance(instance))
            } else {
                Err(Error::Runtime(format!(
                    "{:?} Call only call functions and classes.",
                    paren
                )))
            }
        } else {
            Err(Error::Runtime(format!("Something is very wrong!")))
        }
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Result<Object> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            let left = self.evaluate(left)?;

            // short-circuit logic
            if operator.token_type == TokenType::OR {
                if self.is_truthy(&left) {
                    return Ok(left);
                }
            } else if !self.is_truthy(&left) {
                return Ok(left);
            }

            return self.evaluate(right);
        } else {
            Err(Error::Runtime(format!(
                "visit_logical_expr called for non Expr::Logical enum!"
            )))
        }
    }

    fn visit_get_expr(&mut self, expr: &Expr) -> Result<Object> {

        println!("calling instance.get");
        if let Expr::Get { object, name } = expr {
            let object = self.evaluate(object)?;
            if let Object::Instance(object) = object {
                object.get(name)
            } else {
                Err(Error::Runtime(format!("Only instances have properties.")))
            }
        } else {
            Err(Error::Runtime(format!("Something is very wrong!")))
        }
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_print_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::Print(expr) = statement {
            match self.evaluate(expr) {
                Ok(value) => {
                    println!("{}", value);
                    Ok(())
                }
                Err(err) => Err(err),
            }
        } else {
            // TODO:: this should be a runtime error of sorts?
            Ok(())
        }
    }

    fn visit_expression_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::Expression(expr) = statement {
            match self.evaluate(expr) {
                Ok(value) => Ok(()),
                Err(err) => Err(err),
            }
        } else {
            println!("why are we here?");
            Ok(())
        }
    }

    fn visit_variable_statement(&mut self, statement: &Stmt) -> Result<()> {
        // println!("visit_variable_statement for {:?}", statement);
        if let Stmt::Variable { name, initializer } = statement {
            initializer
                .as_ref()
                .map_or(Ok(Object::None), |init| self.evaluate(init))
                .map(|value| {
                    self.environment
                        .borrow_mut()
                        .define(name.lexeme.clone(), value)
                })
        } else {
            Err(Error::Runtime(format!("this should never happend")))
        }
    }

    fn visit_block_statement(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        self.execute_block(
            statements,
            Rc::new(RefCell::new(Environment::new(&self.environment))),
        )
    }

    fn visit_if_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::If {
            condition,
            else_branch,
            then_branch,
        } = statement
        {
            let value = self.evaluate(condition)?;
            if self.is_truthy(&value) {
                self.execute(then_branch)?;
            } else if let Some(else_branch) = else_branch {
                self.execute(&else_branch)?;
            }
        }

        Ok(())
    }

    fn visit_while_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::While { condition, body } = statement {
            let mut value = self.evaluate(condition)?;
            while self.is_truthy(&value) {
                self.execute(body)?;
                value = self.evaluate(condition)?;
            }
        }

        Ok(())
    }

    fn visit_function_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::Function { name, params, body } = statement {
            let function = Object::Callable(Function::User {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
                closure: Rc::clone(&self.environment), // i guess we need the closure here <_<
            });
            self.environment
                .borrow_mut()
                .define(name.lexeme.clone(), function);
        }
        Ok(())
    }

    fn visit_return_statement(&mut self, statement: &Stmt) -> Result<()> {
        // println!("Hit Return statement");
        // println!("statement = {:?}", statement);
        if let Stmt::Return { keyword, value } = statement {
            let mut return_value = Object::None;
            if let Some(value) = value {
                return_value = self.evaluate(value)?;
            }
            Err(Error::Return {
                value: return_value,
            })
        } else {
            Err(Error::Runtime(format!("this should never happend")))
        }
    }

    fn visit_class_statement(&mut self, statement: &Stmt) -> Result<()> {
        if let Stmt::Class { name, methods } = statement {
            self.environment
                .borrow_mut()
                .define(name.lexeme.clone(), Object::None);

            let class = Object::Class(Class::new(name.lexeme.clone()));

            self.environment.borrow_mut().assign(name, class);

            Ok(())
        } else {
            Err(Error::Runtime(format!("this should never happend")))
        }
    }
}
