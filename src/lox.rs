use std::error;
use std::path::Path;

use crate::errors::Error;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;
use crate::tokens::Token;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Lox {
    had_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false,
            interpreter: Interpreter::new(),
        }
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
            line.clear();
            self.had_error = false;
        }

        Ok(())
    }

    pub fn run_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let source: String = std::fs::read_to_string(path)?;
        self.run(&source);

        if self.had_error {
            Err(Error::Syntax.into())
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
        let mut parser = Parser::new(tokens.clone());
        let statements = parser.parse();

        match statements.and_then(|statements| {
            let mut resolver = Resolver::new(&mut self.interpreter);
            resolver.resolve_statements(&statements)?;
            self.interpreter.interpret(&statements)
        }) {
            Ok(_) => {}
            Err(err) => {
                self.had_error = true;
                println!("ERROR {}", err);
                return;
            }
        }
    }
}

