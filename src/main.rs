// Stopped at https://craftinginterpreters.com/functions.html

#[allow(unused_imports)]
mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod lox;
mod lox_function;
mod parser;
// mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;

use crate::interpreter::Interpreter;
#[allow(unused_imports)]
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let interpreter = Interpreter::new();
    let mut lox_runtime = lox::Lox {
        had_error: false,
        had_runtime_error: false,
        interpreter,
    };
    if args.len() > 2 {
        println!("Usage: jlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        lox_runtime.run_file(&args[1]);
    } else {
        lox_runtime.run_prompt();
    }
    // ast_printer::main();
}

#[cfg(test)]
mod tests {
    #[test]
    fn scan_file() {}
}
