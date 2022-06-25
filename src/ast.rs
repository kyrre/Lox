use crate::tokens::{Literal, Token};
use std::fmt;


#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Unary {
        operator: Token,
        right: Box<Expr>
    },
    Grouping {
        expression: Box<Expr>
    },
    Literal {
        value: Literal,
    },
}

pub trait Visitor<T> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&self, expr: &Expr) -> T;
    fn visit_literal_expr(&self, value: &Literal) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
        }
    }
}

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        expr.accept(self)
    }
}

fn parenthesize(visitor: &dyn Visitor<String>, name: &str, expressions: &Vec<&Expr>) -> String {
    let mut value = String::new();

    value.push('(');
    value.push_str(name);
    value = expressions.iter().fold(value, |mut accu, expr| {
        accu.push(' ');
        accu.push_str(&expr.accept(visitor));
        accu
    });
    value.push(')');
    value
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        parenthesize(self, &operator.lexeme, &vec![left, right])
    }
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        parenthesize(self, &operator.lexeme, &vec![right])
    }
    fn visit_grouping_expr(&self, expr: &Expr) -> String {
        parenthesize(self, "group", &vec![expr])
    }

    fn visit_literal_expr(&self, value: &Literal) -> String {
        value.to_string()
    }
}
