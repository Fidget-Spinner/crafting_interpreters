// Generated by generate_ast.py!

use crate::token::*;

pub trait Expr<T>{
  fn accept(visitor: impl ExprVisitor<T>) -> T;
}
pub trait ExprVisitor<T> {
    fn visit_binary_expr(&self) -> T;
    fn visit_grouping_expr(&self) -> T;
    fn visit_literal_expr(&self) -> T;
    fn visit_unary_expr(&self) -> T;
}
pub struct Binary<T>{left: Box<dyn Expr<T>>, operator: Token, right: Box<dyn Expr<T>>}
impl <T>Expr<T> for Binary<T> {
    fn accept(visitor: impl ExprVisitor<T>) -> T {
        visitor.visit_binary
    }
}

pub struct Grouping<T>{expression: Box<dyn Expr<T>>}
impl <T>Expr<T> for Grouping<T> {
    fn accept(visitor: impl ExprVisitor<T>) -> T {
        visitor.visit_grouping
    }
}

pub struct Literal<T>{value: Literal<T>}
impl <T>Expr<T> for Literal<T> {
    fn accept(visitor: impl ExprVisitor<T>) -> T {
        visitor.visit_literal
    }
}

pub struct Unary<T>{operator: Token, right: Box<dyn Expr<T>>}
impl <T>Expr<T> for Unary<T> {
    fn accept(visitor: impl ExprVisitor<T>) -> T {
        visitor.visit_unary
    }
}

