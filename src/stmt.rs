use crate::expr::RcExpr;
use crate::token::RcToken;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Block {
        statements: Rc<Vec<RcStmt>>,
    },
    Expression {
        expr: RcExpr,
    },
    Function {
        name: RcToken,
        params: Vec<RcToken>,
        body: Rc<Vec<RcStmt>>,
    },
    If {
        condition: RcExpr,
        then_branch: RcStmt,
        else_branch: Option<RcStmt>,
    },
    Print {
        expr: RcExpr,
    },
    Return {
        keyword: RcToken,
        value: RcExpr,
    },
    Var {
        name: RcToken,
        initializer: Option<RcExpr>,
    },
    While {
        condition: RcExpr,
        body: RcStmt,
    },
}

pub type RcStmt = Rc<Stmt>;
