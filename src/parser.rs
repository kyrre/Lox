#![allow(dead_code, unused)]

use crate::ast::Expr;
use crate::errors::{LoxError, Result};
use crate::tokens::{Literal, Token, TokenType};
use std::{error, fmt};

use TokenType::*;


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
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
            self.primary()
        }
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

        // println!("current token: {}", self.peek());
        eprintln!("Expected expression");
        Err(LoxError::Parse {})
    }

    fn consume(&mut self, _type: TokenType, error: &str) -> Result<()> {
        if self.check(_type) {
            self.advance();
            Ok(())
        } else {
            let token = self.peek();
            // todo: refactor this into a separate function or module
            eprintln!("{} at {} {}", token.line, token.token_type, error);
            Err(LoxError::Parse{})
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
