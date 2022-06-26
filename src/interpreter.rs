#![allow(dead_code, unused, non_camel_case_types, non_snake_case)]
use std::f32::MIN;

use crate::ast::{Expr, Visitor};
use crate::tokens::{Token, TokenType::*, Literal};
use crate::errors::Result;


use Literal::{Boolean, String, Number, None as Null};


pub struct Interpreter {}

impl Interpreter {

    pub fn interpret(&self, expr: &Expr) {
        let value = self.evaluate(expr);
        println!("> {:?}", value);
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
}

impl Visitor<Literal> for Interpreter {
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
            (PLUS, Number(left), Number(right)) => {
                Ok(Number(left + right))},
            (PLUS, Literal::String(left), Literal::String(right)) => Ok(Literal::String(left.clone() + right)),

            _ => {
                eprintln!("Error evaluating {:?} {:?} {:?}", left, operator, right);
                Ok(Null)
            }
        }
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<Literal> {
        let right = self.evaluate(right)?;

        match (&operator.token_type, &right) {
            (MINUS, Number(val)) => Ok(Number(-val)),
            (MINUS, _) => {
                eprintln!("Tried to negate operand {:?}", right);
                Ok(Null)
            }
            (BANG, _) => Ok(Boolean(self.is_truthy(right))),
            _ => Ok(Null), // should be unreachable!
        }
    }
    fn visit_grouping_expr(&self, expr: &Expr) -> Result<Literal> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, value: &Literal) -> Result<Literal> {
        Ok(value.clone())
    }
}
