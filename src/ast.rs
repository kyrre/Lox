use crate::errors::Result;
use crate::tokens::{Literal, Token};

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>

    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>
    }
}

pub trait Visitor<T> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Result<T>;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<T>;
    fn visit_grouping_expr(&self, expr: &Expr) -> Result<T>;
    fn visit_literal_expr(&self, value: &Literal) -> Result<T>;
    fn visit_variable_expr(&self, name: &Token) -> Result<T>;
    fn visit_variable_assignment_expr(&self, expr: &Expr) -> Result<T>;
    fn visit_logical_expr(&self, expr: &Expr) -> Result<T>;
    fn visit_call_expr(&self, expr: &Expr) -> Result<T>;

}

impl Expr {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> Result<T> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Variable { name } => visitor.visit_variable_expr(&name),
            Expr::Assign { .. } => visitor.visit_variable_assignment_expr(&self),
            Expr::Logical{..} => visitor.visit_logical_expr(self),
            Expr::Call { .. } => visitor.visit_call_expr(self)
        }
    }
}

//pub struct AstPrinter {}
//
//impl AstPrinter {
//    pub fn print(&self, expr: &Expr) -> Result<String> {
//        expr.accept(self)
//    }
//}
//
//fn parenthesize(
//    visitor: &dyn Visitor<String>,
//    name: &str,
//    expressions: &Vec<&Expr>,
//) -> Result<String> {
//    let mut value = String::new();
//    value.push('(');
//    value.push_str(name);
//    value = expressions.iter().fold(value, |mut accu, expr| {
//        accu.push(' ');
//        if let Ok(res) = &expr.accept(visitor) {
//            accu.push_str(res);
//        }
//        accu
//    });
//    value.push(')');
//    Ok(value)
//}

//impl Visitor<String> for AstPrinter {
    //fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Result<String> {
        //parenthesize(self, &operator.lexeme, &vec![left, right])
    //}
    //fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<String> {
        //parenthesize(self, &operator.lexeme, &vec![right])
    //}
    //fn visit_grouping_expr(&self, expr: &Expr) -> Result<String> {
        //parenthesize(self, "group", &vec![expr])
    //}

    //fn visit_literal_expr(&self, value: &Literal) -> Result<String> {
        //Ok(value.to_string())
    //}

    //fn visit_variable_expr(&self, name: &Token) -> Result<String> {
        //Ok(name.lexeme.clone())
    //}

    //fn visit_variable_assignment_expr(&self, expr: &Expr) -> Result<String> {
        //parenthesize(self, "+", &vec![expr])
    //}
//}
