use crate::expr::Expr;
use crate::token::Token;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Expression {
        expr: Box<Expr>,
    },
    Function {
        name: Box<Token>,
        params: Vec<Box<Token>>,
        body: Vec<Box<Stmt>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expr: Box<Expr>,
    },
    Return {
        keyword: Box<Token>,
        value: Box<Expr>,
    },
    Var {
        name: Box<Token>,
        initializer: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
}
