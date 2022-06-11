#![allow(dead_code, unused)]

use rlox::lox::Lox;
use rlox::tokens::{Token, TokenType};
use std::fmt;
use std::fs;

fn parse_comment(comment: &str) -> String {
    if let Some((_, expect)) = comment.split_once("expect: ") {
        expect.to_string()
    } else {
        "".to_string()
    }
}

fn main() {
    let file = "./tests/test_cases/scanning/identifiers.lox".to_string();

    let mut lox = Lox::new();
    let tokens = lox.run_debug_file(&file);

    let contents = fs::read_to_string(file).unwrap_or(String::new());
    let expected_tokens = contents
        .split('\n')
        .filter(|x| x.starts_with("//"))
        .map(|comment| parse_comment(comment).trim().to_string());

    for (token, expect) in expected_tokens.zip(tokens) {
        assert_eq!(token, expect.to_string());
    }
}
