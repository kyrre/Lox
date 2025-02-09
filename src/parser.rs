#![allow(dead_code, unused)]

use std::{error, fmt};

use crate::ast::Expr;
use crate::errors::{Error, Result};
use crate::statement::Stmt;
use crate::tokens::{Literal, Token, TokenType};

use TokenType::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::<Result<Stmt>>::new();
        while !self.is_at_end() {
            statements.push(self.declaration())
        }

        let statements: Result<Vec<Stmt>> = statements.into_iter().collect();
        statements
    }

    fn declaration(&mut self) -> Result<Stmt> {
        // varDecl  → "var" IDENTIFIER ( "=" expression )? ";" ;

        let res = {
            if self.matches(vec![CLASS]) {
                self.class_declaration()
            } else if self.matches(vec![FUN]) {
                self.function("function")
            } else if self.matches(vec![VAR]) {
                self.var_declaration()
            } else {
                self.statement()
            }
        };

        if res.is_err() {
            self.synchronize();
            Ok(Stmt::Print(Expr::Literal {
                value: Literal::String("called sync".to_string()),
            }))
        } else {
            res
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(IDENTIFIER, "Expect a class name")?;
        self.consume(LEFT_BRACE, "Expect a '{' before class body.")?;

        let mut methods = Vec::new();

        while !self.check(RIGHT_BRACE) && !self.is_at_end() {
            methods.push(Box::new(self.function("method")?));
        }

        self.consume(RIGHT_BRACE, "Expect a '}' after class body.")?;

        Ok(Stmt::Class { name, methods })
    }

    fn function(&mut self, kind: &str) -> Result<Stmt> {
        println!("inside function()");
        let name = self.consume(IDENTIFIER, &format!("Expect {} name.", kind))?;

        self.consume(LEFT_PAREN, &format!("Expect '(' after {} name.", kind))?;

        let mut parameters = Vec::new();
        if !self.check(RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    return Err(Error::Runtime(format!(
                        "{:?} Can't have more than 255 parameters.",
                        self.peek()
                    )));
                }

                parameters.push(self.consume(IDENTIFIER, "Expect parameter name.")?);

                if !self.matches(vec![COMMA]) {
                    break;
                }
            }
        }

        self.consume(RIGHT_PAREN, "Expect ')' after parameters.")?;
        self.consume(LEFT_BRACE, &format!("Expect '{{' before {} body.", kind))?;
        let body = self.block()?;

        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(IDENTIFIER, "Expect variable name.")?;

        let mut initializer: Option<Expr> = None;

        let initializer = if self.matches(vec![EQUAL]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(SEMICOLON, "Expect ';' after variable declaration")?;

        Ok(Stmt::Variable {
            name: name,
            initializer: initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(vec![PRINT]) {
            self.print_statement()
        } else if self.matches(vec![RETURN]) {
            self.return_statement()
        } else if self.matches(vec![WHILE]) {
            self.while_statement()
        } else if self.matches(vec![FOR]) {
            self.for_statement()
        } else if (self.matches(vec![LEFT_BRACE])) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else if (self.matches(vec![IF])) {
            self.if_statement()
        } else {
            self.expr_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous();

        let mut value = None;
        if !self.check(SEMICOLON) {
            value = Some(self.expression()?);
        }

        self.consume(SEMICOLON, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        println!("while statement");
        self.consume(LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(LEFT_PAREN, "Expect a '(' after 'for.'")?;

        let mut initializer = None;

        if self.matches(vec![SEMICOLON]) {
            initializer = None;
        } else if self.matches(vec![VAR]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expr_statement()?);
        }

        let mut condition = None;

        if !self.check(SEMICOLON) {
            condition = Some(self.expression()?);
        }

        self.consume(SEMICOLON, "Expect ';' after loop condition.");

        let mut increment = None;
        if !self.check(RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }

        self.consume(RIGHT_PAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement();

        if let Some(increment) = increment {
            body = Ok(Stmt::Block {
                statements: vec![body?, Stmt::Expression(increment)],
            });
        }

        if let None = condition {
            condition = Some(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }

        if let Some(condition) = condition {
            body = Ok(Stmt::While {
                condition,
                body: Box::new(body?),
            });
        }

        if let Some(initializer) = initializer {
            body = Ok(Stmt::Block {
                statements: vec![initializer, body?],
            });
        }

        // println!("{:?}", body);

        body
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(LEFT_PAREN, "Exepect '(' after 'if'.")?;

        let condition = self.expression()?;

        self.consume(RIGHT_PAREN, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(vec![ELSE]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(RIGHT_BRACE, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression();
        self.consume(SEMICOLON, "Expect ';' after value")?;
        expr.map(Stmt::Print)
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression();
        self.consume(SEMICOLON, "Expect ';' after expression")?;
        expr.map(Stmt::Expression)
    }
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        // assignment → IDENTIFIER "=" assignment | equality ;

        let expr = self.or();

        if (self.matches(vec![EQUAL])) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Ok(Expr::Variable { name }) = expr {
                Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                })
            } else {
                Err(Error::Runtime(format!("Invalid assignment target.")))
            }
        } else {
            expr
        }
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;
        while self.matches(vec![OR]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.matches(vec![AND]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        // equality → comparison ( ( "!=" | "==" ) comparison )* ;
        let mut expr = self.comparison()?;

        while self.matches(vec![BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn comparison(&mut self) -> Result<Expr> {
        // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;

        let mut expr = self.term()?;

        while self.matches(vec![GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // can we rewrite these using, say, a macro?
    // or better just a function pointer
    fn term(&mut self) -> Result<Expr> {
        // term   → factor ( ( "-" | "+" ) factor )* ;

        let mut expr = self.factor()?;

        while self.matches(vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.matches(vec![SLASH, STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches(vec![BANG, MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator: operator,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary();

        loop {
            if self.matches(vec![LEFT_PAREN]) {
                expr = self.finish_call(expr?);
            } else if self.matches(vec![DOT]) {
                let name = self.consume(IDENTIFIER, "Expect property name after '.'.")?;
                expr = Ok(Expr::Get {
                    object: Box::new(expr?),
                    name: name,
                });
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments: Vec<Expr> = Vec::new();

        if !self.check(RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    return Err(Error::Runtime(format!(
                        "{:?} Can't have more than 255 arguments.",
                        self.peek()
                    )));
                }

                arguments.push(self.expression()?);
                if !self.matches(vec![COMMA]) {
                    break;
                }
            }
        }

        let paren = self.consume(RIGHT_PAREN, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr> {
        // primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

        if self.matches(vec![FALSE]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }
        if self.matches(vec![TRUE]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }
        if self.matches(vec![NIL]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }
        if self.matches(vec![STRING, NUMBER]) {
            return Ok(Expr::Literal {
                value: self.previous().literal,
            });
        }
        if self.matches(vec![LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expect ')' after expression.");
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }
        if self.matches(vec![IDENTIFIER]) {
            return Ok(Expr::Variable {
                name: self.previous(),
            });
        }

        // println!("current token: {}", self.peek());
        eprintln!("Expected expression");
        Err(Error::Parse {})
    }

    fn consume(&mut self, _type: TokenType, error: &str) -> Result<Token> {
        if self.check(_type) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            // todo: refactor this into a separate function or module
            eprintln!("{} at {} {}", token.line, token.token_type, error);
            Err(Error::Parse {})
        }
    }

    fn matches(&mut self, types: Vec<TokenType>) -> bool {
        for _type in types {
            if self.check(_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn peek(&mut self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // we assume that this marks the end of a statement!
            if self.previous().token_type == SEMICOLON {
                return;
            }

            // we assume this is the start of a statement
            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }
}
