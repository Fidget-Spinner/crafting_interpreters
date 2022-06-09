use crate::expr::Expr;
use crate::token::RcToken;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Expression {
        expr: Box<Expr>,
    },
    Function {
        name: RcToken,
        params: Vec<RcToken>,
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
        keyword: RcToken,
        value: Box<Expr>,
    },
    Var {
        name: RcToken,
        initializer: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
}
