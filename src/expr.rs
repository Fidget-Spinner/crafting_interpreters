use crate::token::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Assign {
        name: Box<Token>,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Box<Token>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Box<Token>,
        arguments: Vec<Box<Expr>>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: Box<Token>,
        right: Box<Expr>,
    },
    Unary {
        operator: Box<Token>,
        right: Box<Expr>,
    },
    Variable {
        name: Box<Token>,
    },
}
