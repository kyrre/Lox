use crate::ast::Expr;
use crate::errors::Result;
use crate::tokens::Token;

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Variable {
        name: Token,
        initializer: Option<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
    },
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> Result<T> {
        match self {
            Self::Print(_) => visitor.visit_print_statement(self),
            Self::Expression(_) => visitor.visit_expression_statement(self),
            Self::Variable { .. } => visitor.visit_variable_statement(self),
            Self::Block{statements} =>  visitor.visit_block_statement(&statements)
        }
    }
}

pub trait Visitor<T> {
    fn visit_print_statement(&self, statement: &Stmt) -> Result<T>;
    fn visit_expression_statement(&self, statement: &Stmt) -> Result<T>;
    fn visit_variable_statement(&self, statement: &Stmt) -> Result<T>;
    fn visit_block_statement(&mut self, statement: &Vec<Stmt>) -> Result<T>;
}
