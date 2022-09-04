#![allow(dead_code, unused)]
use crate::ast::{Expr, Visitor as ExprVisitor};
use crate::errors::{Error, Result};
use crate::interpreter::Interpreter;
use crate::statement::{self, Stmt, Visitor as StmtVisitor};
use crate::tokens::Token;
use std::collections::HashMap;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::default(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::default());
    }
    pub fn resolve_statements(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements.iter() {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> Result<()> {
        // println!("resolve_statement {:?}", statement);
        statement.accept(self)
    }

    fn resolve_expression(&mut self, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn resolve_function(&mut self, function: &Stmt) -> Result<()> {
        if let Stmt::Function { name, params, body } = function {
            self.begin_scope();

            // parameters
            for param in params.iter() {
                self.declare(param);
                self.define(param);
            }
            self.resolve_statements(body)?;
            self.end_scope();
            Ok(())
        } else {
            Err(Error::Runtime(format!(
                "resolve_function:: should never happend"
            )))
        }
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                // println!("Variable with name {} already declared in this scope.", name.lexeme);
                panic!("aborting");
            }

            // println!("DECLARING variable {:?}", name);
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn resolve_local(&mut self, name: &Token) {
        // println!("resolve_local -- {:?}", name);
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            //println!("{:?}", scope);
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name, i);
                break;
            }
        }
    }

    // fn resolve(&mut self, initializer: &Expr) {}

    fn define(&mut self, name: &Token) {
        // println!("\tDEFINING {:?}", name);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &crate::tokens::Token,
        right: &Expr,
    ) -> crate::errors::Result<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> crate::errors::Result<()> {
        if let Expr::Call {
            callee,
            paren,
            arguments,
        } = expr
        {
            self.resolve_expression(&callee)?;
            for arg in arguments.iter() {
                self.resolve_expression(arg)?;
            }

            Ok(())
        } else {
            Err(Error::Runtime(format!(
                "resolver::visit_call_expr - should never happend"
            )))
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> crate::errors::Result<()> {
        self.resolve_expression(expr)
    }

    fn visit_literal_expr(&mut self, value: &crate::tokens::Literal) -> crate::errors::Result<()> {
        // println!("visit_literal_expr");
        Ok(())
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> crate::errors::Result<()> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            self.resolve_expression(left)?;
            self.resolve_expression(right)
        } else {
            Err(Error::Runtime(format!("should never happend")))
        }
    }

    fn visit_unary_expr(
        &mut self,
        operator: &crate::tokens::Token,
        right: &Expr,
    ) -> crate::errors::Result<()> {
        self.resolve_expression(right)
    }

    fn visit_variable_assignment_expr(&mut self, expr: &Expr) -> crate::errors::Result<()> {
        if let Expr::Assign { name, value } = expr {
            self.resolve_expression(value)?;
            self.resolve_local(name);

            Ok(())
        } else {
            Err(Error::Runtime(format!("should never happen")))
        }
    }

    fn visit_variable_expr(&mut self, name: &crate::tokens::Token) -> crate::errors::Result<()> {
        // println!("visit_varibale_expr:: name = {:?}", name);
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(false) = scope.get(&name.lexeme) {
                return Err(Error::Runtime(format!(
                    "Can't read local variable in its own initializer."
                )));
            }
        }

        self.resolve_local(name);

        Ok(())
    }
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
    fn visit_block_statement(&mut self, statements: &Vec<Stmt>) -> crate::errors::Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_expression_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::Expression(expr) = statement {
            self.resolve_expression(expr)
        } else {
            Err(Error::Runtime(format!("should never happen!")))
        }
    }

    fn visit_function_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::Function { name, params, body } = statement {
            self.declare(name);
            self.define(name);
            self.resolve_function(statement)
        } else {
            Err(Error::Runtime(format!("should never happen!")))
        }
    }

    fn visit_if_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::If {
            condition,
            then_branch,
            else_branch,
        } = statement
        {
            self.resolve_expression(condition)?;
            self.resolve_statement(then_branch)?;

            // this can be rewritten as a map operation i believe
            if let Some(else_branch) = else_branch {
                self.resolve_statement(else_branch)
            } else {
                Ok(())
            }
        } else {
            Err(Error::Runtime(format!("should never happen!")))
        }
    }

    fn visit_print_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::Print(expr) = statement {
            self.resolve_expression(expr)
        } else {
            Err(Error::Runtime(format!("should never happen!")))
        }
    }

    fn visit_return_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::Return { keyword, value } = statement {
            if let Some(expr) = value {
                self.resolve_expression(expr)
            } else {
                Ok(())
            }
        } else {
            Err(Error::Runtime(format!("should never happen!")))
        }
    }

    fn visit_variable_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        // println!("visit_variable_statement {:?}", statement);
        if let Stmt::Variable { name, initializer } = statement {
            self.declare(name);

            // println!("init = {:?}", initializer);

            if let Some(initializer) = initializer {
                self.resolve_expression(initializer);
            }

            self.define(name);

            Ok(())
        } else {
            Err(Error::Runtime(format!("This should never happen!")))
        }
    }

    fn visit_while_statement(&mut self, statement: &Stmt) -> crate::errors::Result<()> {
        if let Stmt::While { condition, body } = statement {
            self.resolve_expression(condition)?;
            self.resolve_statement(body)
        } else {
            Err(Error::Runtime(format!("This should never happen!")))
        }
    }
}