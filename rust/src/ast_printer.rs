use crate::expr::*;
use crate::token::*;
use crate::token_type::TokenType;
use std::rc::Rc;

#[allow(dead_code)]
pub fn main() {
    let expression = Rc::from(Expr::Binary {
        left: Rc::from(Expr::Unary {
            operator: Rc::new(Token::new(
                TokenType::MINUS,
                "-".as_bytes().to_vec(),
                Literal::NIL,
                1,
            )),
            right: Rc::from(Expr::Literal(Literal::NUMBER(123.0))),
        }),
        operator: Rc::new(Token::new(
            TokenType::STAR,
            "*".as_bytes().to_vec(),
            Literal::NIL,
            1,
        )),
        right: Rc::from(Expr::Grouping(Rc::from(Expr::Literal(Literal::NUMBER(
            45.67,
        ))))),
    });
    print!("{}", ast_to_string(expression));
}

pub fn ast_to_string(expr: RcExpr) -> String {
    match &*expr {
        Expr::Assign { name, value: _ } => name.lexeme.clone(),
        Expr::Binary {
            left,
            operator,
            right,
        } => parenthesize(
            operator.lexeme.clone(),
            vec![Rc::clone(left), Rc::clone(right)],
        ),
        Expr::Call {
            callee: _,
            paren: _,
            arguments,
        } => parenthesize(String::from("call"), arguments.clone()),
        Expr::Grouping(expr) => parenthesize(String::from("group"), vec![Rc::clone(expr)]),
        Expr::Literal(literal) => literal.to_string(),
        Expr::Logical {
            left,
            operator,
            right,
        } => parenthesize(
            operator.lexeme.clone(),
            vec![Rc::clone(left), Rc::clone(right)],
        ),
        Expr::Unary { operator, right } => {
            parenthesize(operator.lexeme.clone(), vec![Rc::clone(right)])
        }
        Expr::Variable { name } => name.lexeme.clone(),
    }
}

fn parenthesize(name: String, exprs: Vec<RcExpr>) -> String {
    let mut builder: String = String::with_capacity(2 + exprs.len() * 2);
    builder.push('(');
    builder.push_str(&name);
    for expr in exprs {
        builder.push(' ');
        builder.push_str(&ast_to_string(expr));
    }
    builder.push(')');
    builder
}
