use crate::ast::Expr;
use crate::errors::Result;
use crate::tokens::Token;
use crate::object::Object;

#[derive(Clone, Debug)]
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
    If {
       condition: Expr,
       then_branch: Box<Stmt>,
       else_branch: Option<Box<Stmt>>
    },
    While {
        condition: Expr,
        body: Box<Stmt>
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>
    },
    Return {
        keyword: Token,
        value: Option<Expr>
    }
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> Result<T> {
        match self {
            Self::Print(_) => visitor.visit_print_statement(self),
            Self::Expression(_) => visitor.visit_expression_statement(self),
            Self::Variable { .. } => visitor.visit_variable_statement(self),
            Self::Block { statements } => visitor.visit_block_statement(&statements),
            Self::If{..} => visitor.visit_if_statement(self),
            Self::While {..} => visitor.visit_while_statement(self),
            Self::Function { name, params, body } => visitor.visit_function_statement(self),
            Self::Return { keyword, value } => visitor.visit_return_statement(self)

        }
    }
}

pub trait Visitor<T> {
    fn visit_print_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_expression_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_variable_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_block_statement(&mut self, statement: &Vec<Stmt>) -> Result<T>;
    fn visit_if_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_while_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_function_statement(&mut self, statement: &Stmt) -> Result<T>;
    fn visit_return_statement(&mut self, statement: &Stmt) -> Result<T>;
}
