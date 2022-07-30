use std::collections::HashMap;

use crate::tokens::TokenType::{self, *};
use crate::tokens::{Literal, Token};

#[derive(Debug, Default)]
pub struct Scanner<'a> {
    source: Vec<char>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: String) -> Self {
        let keywords = HashMap::from([
            ("and", AND),
            ("class", CLASS),
            ("else", ELSE),
            ("false", FALSE),
            ("for", FOR),
            ("fun", FUN),
            ("if", IF),
            ("nil", NIL),
            ("or", OR),
            ("print", PRINT),
            ("return", RETURN),
            ("super", SUPER),
            ("this", THIS),
            ("true", TRUE),
            ("var", VAR),
            ("while", WHILE),
        ]);

        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current = self.current + 1;
        ch
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, Literal::None);
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Literal) {
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let token = Token::new(token_type, text, literal, self.line);
        self.tokens.push(token);
    }

    fn is_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current = self.current + 1;
        true
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            ',' => self.add_token(COMMA),
            '.' => self.add_token(DOT),
            '-' => self.add_token(MINUS),
            '+' => self.add_token(PLUS),
            ';' => self.add_token(SEMICOLON),
            '*' => self.add_token(STAR),

            // Operators
            '!' => {
                let token = if self.is_match('=') { BANG_EQUAL } else { BANG };
                self.add_token(token);
            }

            '=' => {
                let token = if self.is_match('=') {
                    EQUAL_EQUAL
                } else {
                    EQUAL
                };
                self.add_token(token);
            }

            '<' => {
                let token = if self.is_match('=') { LESS_EQUAL } else { LESS };
                self.add_token(token);
            }

            '>' => {
                let token = if self.is_match('=') {
                    GREATER_EQUAL
                } else {
                    GREATER
                };
                self.add_token(token);
            }

            '/' => {
                // this line is a comment!
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(SLASH);
                }
            }

            // whitespace characters
            '\r' | '\t' | ' ' => {}
            '\n' => self.line = self.line + 1,

            // string literals
            '"' => {
                self.string();
            }

            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    eprintln!("Unexpected character: {} at line: {}", c, self.line);
                }
            }
        };
    }
    fn substring(&self, start: usize, stop: usize) -> String {
        self.source[start..stop].iter().collect::<String>()
    }

    fn identifier(&mut self) {
        while self.source[self.current].is_alphanumeric() || self.source[self.current] == '_' {
            self.advance();
        }

        let text = self.substring(self.start, self.current);

        // https://stackoverflow.com/questions/65549983/trait-borrowstring-is-not-implemented-for-str
        if self.keywords.contains_key(&text as &str) {
            let token_type = self.keywords.get(&text as &str).unwrap();
            self.add_token(token_type.clone());
        } else {
            self.add_token(IDENTIFIER);
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line = self.line + 1
            };
            self.advance();
        }

        if self.is_at_end() {
            eprintln!("Unterminated string!");
            return;
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect::<String>();
        self.add_token_literal(STRING, Literal::String(value));
    }

    pub fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let number = value.parse::<f64>().unwrap();

        self.add_token_literal(NUMBER, Literal::Number(number))
    }

    pub fn is_digit(&self, ch: char) -> bool {
        ch.is_digit(10)
    }

    pub fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    pub fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        let token = Token::new(EOF, "".to_string(), Literal::None, self.line);
        self.tokens.push(token);
        self.tokens.clone()
    }
}
