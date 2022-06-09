use crate::token::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Assign {
        name: RcToken,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: RcToken,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: RcToken,
        arguments: Vec<Box<Expr>>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: RcToken,
        right: Box<Expr>,
    },
    Unary {
        operator: RcToken,
        right: Box<Expr>,
    },
    Variable {
        name: RcToken,
    },
}
