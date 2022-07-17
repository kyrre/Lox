use std::env;
use std::error;
use std::process::exit;

use rlox::lox::Lox;

const EX_USAGE: i32 = 64;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut lox = Lox::new();

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
