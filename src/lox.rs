use std::fmt;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::Write;
use std::process;
use std::rc::Rc;

// use crate::ast_printer::ast_to_string;
// use crate::expr::Expr;
use crate::interpreter::{ExprValue, SharedInterpreter};
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;
use crate::token::RcToken;
use crate::token_type::TokenType;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LoxError<T: Display> {
    ScanError { line: usize, message: T },
    ParseError { token: RcToken, message: T },
    RuntimeError { token: RcToken, message: T },
    ReturnValue { value: Rc<ExprValue> },
}

// for debugging only
impl<T: Display> Display for LoxError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxError::ScanError { line, message } => {
                let location = 0;
                write!(f, "[line {}] Error {}: {}", line, location, message)
            }
            LoxError::RuntimeError { token, message } => {
                let location = 0;
                write!(f, "[line {}] Error {}: {}", token.line, location, message)
            }
            // LoxError::RuntimeError { expr, message } => match expr {
            //     Expr::Binary { left, operator, .. } | Expr::Unary { operator, .. } => write!(
            //         f,
            //         "[line {} token {}] Error {}",
            //         operator.line,
            //         operator.literal.to_string(),
            //         message
            //     ),
            //     _ => unreachable!(),
            // },
            LoxError::ParseError { token, message } => {
                write!(
                    f,
                    "[line {} token {:?}] Error {}",
                    token.line, token, message
                )
            }
            LoxError::ReturnValue { value } => write!(f, "Return {:?}", value),
        }
    }
}

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
    pub interpreter: SharedInterpreter,
}

impl Lox {
    pub fn run_file(&mut self, path: &String) {
        let contents = fs::read_to_string(path)
            .expect("Couldn't read file.")
            .into_bytes();
        self.run(contents);
        if self.had_error {
            process::exit(65);
        }
        if self.had_runtime_error {
            process::exit(70);
        }
    }
    pub fn run_prompt(&mut self) {
        println!("Lox tree-walk interpreter");
        loop {
            print!("> ");
            io::stdout().flush().expect("Couldn't flush print buffer");
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");
            // println!();
            if line.is_empty() {
                println!("Exit");
                break;
            }
            self.run(line.into_bytes());
            self.had_error = false;
        }
    }
    fn run(&mut self, source: Vec<u8>) {
        let mut scanner = Scanner::new(source);
        if let Err(err) = scanner.scan_tokens() {
            self.error(err);
            return;
        }

        let tokens = scanner.tokens;
        let mut parser = Parser::new(self, tokens);
        let res = parser.parse();
        if let Err(e) = res {
            self.error(e);
            return;
        }
        let expr = res.unwrap();
        let mut resolver = Resolver::new(&self.interpreter);
        if let Err(e) = resolver.resolve_statements(&expr) {
            self.error(e);
            return;
        }
        let res = self.interpreter.borrow_mut().interpret(expr);
        // println!("{}", ast_to_string(Box::new(expr)))
        if let Err(e) = res {
            self.error(e)
        }
    }
    pub fn error<T: Display>(&mut self, err: LoxError<T>) {
        match err {
            LoxError::ScanError { line, message } => self.report(line, &"", &message),
            LoxError::RuntimeError { token, message } => self.error_token(token, &message),
            // LoxError::RuntimeError { expr, message } => self.error_runtime(expr, &message),
            LoxError::ParseError { token, message } => self.error_token(token, &message),
            LoxError::ReturnValue { value: _ } => unreachable!("Return outside of function?"),
        }
    }
    fn report<T: Display, U: Display>(&mut self, line: usize, location: &U, message: &T) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }

    fn error_token<T: Display>(&mut self, token: RcToken, message: &T) {
        if matches!(token.type_, TokenType::EOF) {
            self.report(token.line, &"at end", message);
        } else {
            self.report(token.line, &format!("at '{}'", token.lexeme), message);
        }
    }

    // fn error_runtime<T: Display>(&mut self, expr: Expr, message: &T) {
    //     self.had_runtime_error = true;
    //     match expr {
    //         Expr::Binary {
    //             left,
    //             operator,
    //             right,
    //         }
    //         | Expr::Unary { operator, right } => {
    //             self.report(operator.line, &operator.literal.to_string(), message)
    //         }
    //         _ => unreachable!("Unknown operator encountered in runtimeerror"),
    //     }
    // }
}
