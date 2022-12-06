use crate::environment::Environment;
use crate::interpreter::{ExprValue, ExprValueResult, LoxCallable};
use crate::lox::LoxError;
use crate::stmt::{RcStmt, Stmt};
use crate::token::Literal;
use crate::Interpreter;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub declaration: RcStmt,
    pub closure: Rc<RefCell<Environment>>,
}
impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        match self.declaration.borrow() {
            Stmt::Function {
                name: _,
                params,
                body: _,
            } => params.len(),
            _ => unreachable!("Non-function statement in function call?"),
        }
    }
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Rc<ExprValue>>,
    ) -> ExprValueResult {
        let environment = Rc::clone(&self.closure);
        match self.declaration.borrow() {
            Stmt::Function {
                name: _,
                params,
                body,
            } => {
                // Copy args into our environment.
                for i in 0..params.len() {
                    environment
                        .borrow_mut()
                        .define(params[i].lexeme.clone(), Some(Rc::clone(&arguments[i])))
                }
                return match interpreter.execute_block(Rc::clone(body), environment) {
                    Err(LoxError::ReturnValue { value }) => Ok(value),
                    Err(e) => Err(e),
                    _ => Ok(Rc::from(ExprValue::Literal(Literal::NIL))),
                };
                // return Ok(Rc::from(ExprValue::Literal(Literal::BOOL(true))));
            }
            _ => unreachable!("Non-function statement in function call?"),
        }
    }
    fn to_string(&self) -> String {
        match self.declaration.borrow() {
            Stmt::Function {
                name,
                params: _,
                body: _,
            } => {
                format!("<fn {} >", name.lexeme)
            }
            _ => unreachable!("Non-function statement in function call?"),
        }
    }
}
