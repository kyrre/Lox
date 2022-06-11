#![allow(dead_code, unused)]

use std::env;
use std::error;
use std::io;
use std::path::Path;
use std::process::exit;

use rlox::lox;

const EX_USAGE: i32 = 64;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

use rlox::ast::{AstPrinter, Expr};
use rlox::tokens::{Literal, Token, TokenType};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut lox = lox::Lox::new();

    if args.len() > 2 {
        eprintln!("Usage: rlox [script]");
        exit(EX_USAGE);
    } else if args.len() == 2 {
        lox.run_file(&args[1])?;
    } else {
        lox.run_prompt()?;
    }

    Ok(())
}
