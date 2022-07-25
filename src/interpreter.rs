#![allow(dead_code, unused, non_camel_case_types, non_snake_case)]
use std::cell::RefCell;
use std::f32::MIN;

use std::rc::Rc;
use crate::ast::{Expr, Visitor as ExprVisitor};
use crate::environment::{self, Environment};
use crate::errors::{Error, Result};
use crate::statement::{self, Stmt, Visitor as StmtVisitor};
use crate::tokens::{Literal, Token, TokenType::*};

use Literal::{Boolean, None as Null, Number, String};

#[derive(Default, Debug)]
pub struct Interpreter {
    // lets create multiple owners to 
    environment: Rc<RefCell<Environment>>
}

impl Interpreter {
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        statements
            .iter()
            .try_for_each(|statement| self.execute(statement))
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<Literal> {
        expr.accept(self)
    }

    pub fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Null => false,
            Boolean(value) => value,
            _ => false,
        }
    }

    pub fn execute(&mut self, statement: &Stmt) -> Result<()> {
        statement.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        let result = statements.iter().try_for_each(|statement| self.execute(statement));
        self.environment = previous;
        result?;

        Ok(())
    }
}

impl ExprVisitor<Literal> for Interpreter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Result<Literal> {
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
                println!("off the rails!");
                Err(Error::Runtime(format!(
                    "Error evaluating {} {} {}",
                    left, operator, right
                )))
            }
        }
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<Literal> {
        let right = self.evaluate(right)?;

        match (&operator.token_type, &right) {
            (MINUS, Number(val)) => Ok(Number(-val)),
            (MINUS, _) => Err(Error::Runtime(format!(
                "Tried to negate invalid operand {:?}",
                right
            ))),
            (BANG, _) => Ok(Boolean(self.is_truthy(right))),
            _ => Err(Error::Runtime(format!(
                "Invalid unary expr: {:?} {:?}",
                operator, right,
            ))),
        }
    }
    fn visit_grouping_expr(&self, expr: &Expr) -> Result<Literal> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, value: &Literal) -> Result<Literal> {
        Ok(value.clone())
    }

    fn visit_variable_expr(&self, name: &Token) -> Result<Literal> {
        self.environment.borrow().get(name)
    }

    fn visit_variable_assignment_expr(&self, expr: &Expr) -> Result<Literal> {
        if let Expr::Assign { name, value: expr } = expr {
            let value = self.evaluate(expr)?;
            self.environment.borrow_mut().assign(name, value)
        } else {
            Err(Error::Runtime(format!("Something is very wrong!")))
        }
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_print_statement(&self, statement: &Stmt) -> Result<()> {
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

    fn visit_expression_statement(&self, statement: &Stmt) -> Result<()> {
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

    fn visit_variable_statement(&self, statement: &Stmt) -> Result<()> {
        if let Stmt::Variable { name, initializer } = statement {
            initializer
                .as_ref()
                .map_or(Ok(Literal::None), |init| self.evaluate(init))
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
            Rc::new(RefCell::new(Environment::new(&self.environment)))); 
        Ok(()) }
}
