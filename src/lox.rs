use std::error;
use std::fmt;
use std::path::Path;

use crate::scanner::Scanner;
use crate::tokens::Token;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct SyntaxError;

impl error::Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid Lox code!")
    }
}

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox { had_error: false }
    }

    pub fn error(&self, line: u64, message: &str) {
        self.report(line, " ", message);
    }

    fn report(&self, line: u64, _where: &str, message: &str) {
        eprintln!("[line {} ] Error {}  : {}", line, _where, message);
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        let mut line = String::new();
        let input = std::io::stdin();
        while let Ok(n) = input.read_line(&mut line) {
            if n == 0 {
                break;
            }
            self.run(&line);
            self.had_error = false;
        }

        Ok(())
    }

    pub fn run_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let source: String = std::fs::read_to_string(path)?;
        self.run(&source);

        if self.had_error {
            Err(SyntaxError.into())
        } else {
            Ok(())
        }
    }

    pub fn run_debug_file<P: AsRef<Path>>(&mut self, path: P) -> Vec<Token> {
        let source: String = std::fs::read_to_string(path).unwrap_or(String::new());

        let mut scanner = Scanner::new(source.clone());
        let tokens = scanner.scan_tokens();

        tokens
    }

    pub fn run(&mut self, s: &String) {
        let mut scanner = Scanner::new(s.clone());
        let tokens = scanner.scan_tokens();

        for token in tokens {
            println!("{}", token);
        }
    }
}
