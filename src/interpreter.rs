#![allow(dead_code, unused, non_camel_case_types, non_snake_case)]
use std::f32::MIN;

use crate::ast::{Expr, Visitor};
use crate::tokens::{Literal, Token, TokenType::*};

use Literal::{Boolean, Number};

pub struct Interpreter {}

impl Interpreter {

    pub fn interpret(&self, expr: &Expr) {

        println!("{:?}", expr);
        let value = self.evaluate(expr);
        println!("> {}", value);
    }
    pub fn evaluate(&self, expr: &Expr) -> Literal {
        expr.accept(self)
    }

    pub fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Literal::None => false,
            Literal::Boolean(value) => value,
            _ => false,
        }
    }
}

impl Visitor<Literal> for Interpreter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Literal {
        println!("hello from visit_binary_expr");
        let left = self.evaluate(left);
        let right = self.evaluate(right);

        match (&operator.token_type, &left, &right) {
            (GREATER, Number(left), Number(right)) => Boolean(left > right),
            (GREATER_EQUAL, Number(left), Number(right)) => Boolean(left >= right),
            (LESS, Number(left), Number(right)) => Boolean(left < right),
            (LESS_EQUAL, Number(left), Number(right)) => Boolean(left <= right),

            (BANG_EQUAL, left, right) => Boolean(left != right),
            (EQUAL_EQUAL, left, right) => Boolean(left == right),

            (MINUS, Number(left), Number(right)) => Number(left - right),
            (SLASH, Number(left), Number(right)) => Number(left / right),
            (STAR, Number(left), Number(right)) => Number(left * right),
            (PLUS, Number(left), Number(right)) => {
                println!("should hit this arm");
                Number(left + right)},
            (PLUS, Literal::String(left), Literal::String(right)) => Literal::String(left.clone() + right),

            _ => {
                eprintln!("Error evaluating {:?} {:?} {:?}", left, operator, right);
                Literal::None
            }
        }
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Literal {
        let right = self.evaluate(right);

        match (&operator.token_type, &right) {
            (MINUS, Number(val)) => Number(-val),
            (MINUS, _) => {
                eprintln!("Tried to negate operand {:?}", right);
                Literal::None
            }
            (BANG, _) => Boolean(self.is_truthy(right)),
            _ => Literal::None, // should be unreachable!
        }
    }
    fn visit_grouping_expr(&self, expr: &Expr) -> Literal {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, value: &Literal) -> Literal {
        value.clone()
    }
}
