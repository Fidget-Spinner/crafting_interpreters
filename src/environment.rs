use crate::interpreter::ExprValue;
use crate::lox::LoxError;
use crate::token::{Literal, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<ExprValue>>,
}
type OptionExprValue = Option<Rc<ExprValue>>;

macro_rules! insert {
    ($self:ident, $name:expr, $value:ident) => {
        if let Some(v) = $value {
            $self.values.insert($name, v);
        } else {
            $self
                .values
                .insert($name, Rc::new(ExprValue::Literal(Literal::NIL)));
        }
    };
}

impl Environment {
    pub fn new(enclosing: Option<&Rc<RefCell<Environment>>>) -> Self {
        Environment {
            enclosing: {
                if let Some(e) = enclosing {
                    Some(Rc::clone(&e))
                } else {
                    None
                }
            },
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: String, value: OptionExprValue) {
        insert!(self, name, value);
    }
    pub fn get(&self, name: &Token) -> Result<Rc<ExprValue>, LoxError<String>> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(Rc::clone(self.values.get(&name.lexeme).unwrap()));
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(LoxError::RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
    pub fn assign(&mut self, name: &Token, value: OptionExprValue) -> Result<(), LoxError<String>> {
        if self.values.contains_key(&name.lexeme) {
            insert!(self, name.lexeme.to_owned(), value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            Rc::clone(enclosing).borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(LoxError::RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
}
