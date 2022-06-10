use crate::environment::Environment;
use crate::expr::{Expr, RcExpr};
use crate::lox::LoxError;
use crate::lox_function::LoxFunction;
use crate::stmt::{RcStmt, Stmt};
use crate::token::*;
use crate::token_type::TokenType;
use dyn_clone::{clone_trait_object, DynClone};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum ExprValue {
    Literal(Literal),
    LoxCallable(Rc<dyn LoxCallable>),
}

impl ExprValue {
    fn get_number(&self) -> Option<f64> {
        match self {
            ExprValue::Literal(Literal::NUMBER(f)) => Some(*f),
            _ => None,
        }
    }
    fn get_string(&self) -> Option<&String> {
        match self {
            ExprValue::Literal(Literal::STRING(s)) => Some(s),
            _ => None,
        }
    }
}

impl PartialEq for ExprValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ExprValue::Literal(l1) => match other {
                ExprValue::Literal(l2) => l1 == l2,
                _ => false,
            },
            ExprValue::LoxCallable(c1) => match other {
                ExprValue::LoxCallable(c2) => std::ptr::eq(c1, c2),
                _ => false,
            },
        }
    }
}

pub trait LoxCallable: Debug + DynClone {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Rc<ExprValue>>)
        -> ExprValueResult;
    fn to_string(&self) -> String;
}

clone_trait_object!(LoxCallable);

impl PartialEq for dyn LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self, &other)
    }
}

// impl Debug for dyn LoxCallable {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("LoxCallable")
//             // .field("arity", &(self.arity as *const()))
//             // .field("call", &(self.call as *const()))
//             // .field("call", &(self.to_string as *const()))
//             .finish()
//     }
// }

pub type ExprValueResult = Result<Rc<ExprValue>, LoxError<String>>;
pub type VoidResult = Result<(), LoxError<String>>;

macro_rules! operand_err {
    ($operator:tt) => {
        Err(LoxError::RuntimeError {
            token: Rc::clone(&$operator),
            message: format!("{:?} operands must be a number(s)", $operator.type_),
        })
    };
}

// BUILTINS

#[derive(Clone, Debug)]
struct Clock();
impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Rc<ExprValue>>,
    ) -> ExprValueResult {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time is broken");
        Ok(Rc::from(ExprValue::Literal(Literal::NUMBER(
            (duration.as_secs() as f64) + (duration.subsec_nanos() as f64) * 1e-9,
        ))))
    }
    fn to_string(&self) -> String {
        String::from("<native fn>")
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new(None);
        globals.define(
            String::from("clock"),
            Some(Rc::from(ExprValue::LoxCallable(Rc::from(Clock())))),
        );
        let global_env = Rc::from(RefCell::new(globals));
        Interpreter {
            environment: Rc::clone(&global_env),
            globals: global_env,
        }
    }
    pub fn interpret(&mut self, statements: Vec<RcStmt>) -> VoidResult {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }
    fn execute(&mut self, stmt: RcStmt) -> VoidResult {
        match &*stmt {
            Stmt::Block { statements } => {
                self.execute_block(
                    Rc::clone(statements),
                    Rc::from(RefCell::new(Environment::new(Some(&self.environment)))),
                )?;
            }
            Stmt::Expression { expr } => {
                self.evaluate(Rc::clone(expr))?;
            }
            Stmt::Function {
                ref name,
                params: _,
                body: _,
            } => {
                let name_copy = name.lexeme.to_owned();
                let function = LoxFunction {
                    declaration: stmt,
                    closure: Rc::clone(&self.environment),
                };
                self.environment.borrow_mut().define(
                    name_copy,
                    Some(Rc::from(ExprValue::LoxCallable(Rc::from(function)))),
                );
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if Interpreter::is_truthy(&self.evaluate(Rc::clone(condition))?) {
                    self.execute(Rc::clone(then_branch))?;
                } else if let Some(els) = else_branch {
                    self.execute(Rc::clone(els))?;
                }
            }
            Stmt::Print { expr } => {
                let value = self.evaluate(Rc::clone(expr))?;
                println!("{}", Interpreter::stringify(value));
            }
            Stmt::Return { keyword: _, value } => {
                return Err(LoxError::ReturnValue {
                    value: self.evaluate(Rc::clone(value))?,
                });
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;
                if let Some(expr) = initializer {
                    value = Some(self.evaluate(Rc::clone(expr))?);
                }
                (*self.environment)
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), value);
            }
            Stmt::While { condition, body } => {
                while Interpreter::is_truthy(&self.evaluate(Rc::clone(condition))?) {
                    self.execute(Rc::clone(body))?;
                }
            }
        }
        Ok(())
    }
    pub fn execute_block(
        &mut self,
        statements: Rc<Vec<RcStmt>>,
        environment: Rc<RefCell<Environment>>,
    ) -> VoidResult {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;
        for statement in statements.iter() {
            if let Err(e) = self.execute(Rc::clone(statement)) {
                self.environment = previous;
                return Err(e);
            }
        }
        self.environment = previous;
        Ok(())
    }
    fn evaluate(&mut self, expr: RcExpr) -> ExprValueResult {
        match &*expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate(Rc::clone(value))?;
                self.environment
                    .borrow_mut()
                    .assign(&name, Some(Rc::clone(&value)))?;
                Ok(value)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => self.interpret_expr_binary(Rc::clone(left), Rc::clone(operator), Rc::clone(right)),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let eval_callee = self.evaluate(Rc::clone(callee))?;

                let mut eval_arguments: Vec<Rc<ExprValue>> = Vec::with_capacity(arguments.len());
                let arg_len = arguments.len();
                for argument in arguments.into_iter() {
                    eval_arguments.push(self.evaluate(Rc::clone(argument))?);
                }
                let function = match &*eval_callee.borrow() {
                    ExprValue::LoxCallable(function) => function.clone(),
                    _ => {
                        return Err(LoxError::RuntimeError {
                            token: Rc::clone(paren),
                            message: String::from("Can only call functions and classes."),
                        });
                    }
                };
                let arity = function.arity();
                if arg_len != arity {
                    return Err(LoxError::RuntimeError {
                        token: Rc::clone(paren),
                        message: format!("Expected {} arguments but got {}.", arity, arg_len),
                    });
                }
                Ok(function.call(self, eval_arguments)?)
            }
            Expr::Grouping(expr) => self.evaluate(Rc::clone(expr)),
            Expr::Literal(literal) => Ok(Rc::from(ExprValue::Literal(literal.clone()))),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(Rc::clone(left))?;
                if matches!(operator.type_, TokenType::OR) {
                    if Interpreter::is_truthy(&left) {
                        return Ok(Rc::clone(&left));
                    }
                // AND operation
                } else {
                    if !Interpreter::is_truthy(&left) {
                        return Ok(Rc::clone(&left));
                    }
                }
                Ok(self.evaluate(Rc::clone(right))?)
            }
            Expr::Unary { operator, right } => {
                self.interpret_expr_unary(Rc::clone(operator), Rc::clone(right))
            }
            Expr::Variable { name } => self.environment.borrow_mut().get(&name),
        }
    }
    fn interpret_expr_unary(&mut self, operator: RcToken, right: RcExpr) -> ExprValueResult {
        let res = self.evaluate(right)?;
        return match operator.type_ {
            TokenType::MINUS => {
                if let Some(num) = res.get_number() {
                    return Ok(Rc::from(ExprValue::Literal(Literal::NUMBER(-num))));
                }
                return operand_err!(operator);
            }
            TokenType::BANG => Ok(Rc::from(ExprValue::Literal(Literal::BOOL(
                !Interpreter::is_truthy(&res),
            )))),
            _ => unreachable!("Invalid unary operator"),
        };
    }
    fn interpret_expr_binary(
        &mut self,
        left: RcExpr,
        operator: RcToken,
        right: RcExpr,
    ) -> ExprValueResult {
        let res_left = self.evaluate(left)?;
        let res_right = self.evaluate(right)?;
        macro_rules! binary_op_numeric_generic {
            ($op:tt, $type_:tt) => {
                if let (Some(num_left), Some(num_right)) = (res_left.get_number(), res_right.get_number()) {
                    return Ok(Rc::from(ExprValue::Literal(Literal::$type_(num_left $op num_right))));
                }
            };
        }
        macro_rules! binary_num_op {
            ($op:tt) => {
                binary_op_numeric_generic!($op, NUMBER)
            };
        }
        macro_rules! binary_bool_op {
            ($op:tt) => {
                binary_op_numeric_generic!($op, BOOL)
            };
        }
        match operator.type_ {
            TokenType::GREATER => {
                binary_bool_op!(>);
                return operand_err!(operator);
            }
            TokenType::GREATER_EQUAL => {
                binary_bool_op!(>=);
                return operand_err!(operator);
            }
            TokenType::LESS => {
                binary_bool_op!(<);
                return operand_err!(operator);
            }
            TokenType::LESS_EQUAL => {
                binary_bool_op!(<=);
                return operand_err!(operator);
            }
            TokenType::BANG_EQUAL => Ok(Rc::from(ExprValue::Literal(Literal::BOOL(
                res_left != res_right,
            )))),
            TokenType::EQUAL_EQUAL => Ok(Rc::from(ExprValue::Literal(Literal::BOOL(
                res_left == res_right,
            )))),
            TokenType::MINUS => {
                binary_num_op!(-);
                return operand_err!(operator);
            }
            TokenType::PLUS => {
                binary_num_op!(+);
                if let (Some(str_left), Some(str_right)) =
                    (res_left.get_string(), res_right.get_string())
                {
                    return Ok(Rc::from(ExprValue::Literal(Literal::STRING(
                        str_left.to_owned() + str_right,
                    ))));
                }
                return Err(LoxError::RuntimeError {
                    token: Rc::clone(&operator),
                    message: format!("{:?} operand must be numbers or strings", operator.type_),
                });
            }
            TokenType::SLASH => {
                binary_num_op!(/);
                return operand_err!(operator);
            }
            TokenType::STAR => {
                binary_num_op!(*);
                return operand_err!(operator);
            }
            _ => unreachable!("invalid binary operator"),
        }
    }
    fn is_truthy(expr_value: &Rc<ExprValue>) -> bool {
        match expr_value.borrow() {
            ExprValue::Literal(Literal::NIL) => false,
            ExprValue::Literal(Literal::BOOL(b)) => b != &false,
            _ => true,
        }
    }
    fn stringify(object: Rc<ExprValue>) -> String {
        match object.borrow() {
            ExprValue::Literal(l) => l.to_string(),
            ExprValue::LoxCallable(c) => c.to_string(),
        }
    }
}
