use crate::token::*;
use std::rc::Rc;

#[derive(PartialEq, Hash, Clone, Debug, Eq)]
pub enum Expr {
    Assign {
        name: RcToken,
        value: RcExpr,
    },
    Binary {
        left: RcExpr,
        operator: RcToken,
        right: RcExpr,
    },
    Call {
        callee: RcExpr,
        paren: RcToken,
        arguments: Vec<RcExpr>,
    },
    Grouping(RcExpr),
    Literal(Literal),
    Logical {
        left: RcExpr,
        operator: RcToken,
        right: RcExpr,
    },
    Unary {
        operator: RcToken,
        right: RcExpr,
    },
    Variable {
        name: RcToken,
    },
}

pub type RcExpr = Rc<Expr>;
