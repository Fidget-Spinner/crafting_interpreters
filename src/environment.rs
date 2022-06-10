use crate::interpreter::ExprValue;
use crate::lox::LoxError;
use crate::token::{Literal, RcToken};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<ExprValue>>,
}
type OptionExprValue = Option<Rc<ExprValue>>;

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
        if let Some(v) = value {
            self.values.insert(name, v);
        } else {
            self.values
                .insert(name, Rc::from(ExprValue::Literal(Literal::NIL)));
        }
    }
    pub fn get(&self, name: &RcToken) -> Result<Rc<ExprValue>, LoxError<String>> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(Rc::clone(self.values.get(&name.lexeme).unwrap()));
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(LoxError::RuntimeError {
            token: Rc::clone(name),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
    pub fn assign(
        &mut self,
        name: &RcToken,
        value: OptionExprValue,
    ) -> Result<(), LoxError<String>> {
        if self.values.contains_key(&name.lexeme) {
            let val = self.values.get_mut(&name.lexeme).unwrap();
            *val = if let Some(v) = value {
                v
            } else {
                Rc::from(ExprValue::Literal(Literal::NIL))
            };
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            Rc::clone(enclosing).borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(LoxError::RuntimeError {
            token: Rc::clone(name),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
}
