use crate::expr::{Expr, RcExpr};
use crate::interpreter::SharedInterpreter;
use crate::lox::LoxError;
use crate::stmt::{RcStmt, Stmt};
use crate::token::{Literal, RcToken};
use std::collections::HashMap;
use std::rc::Rc;

type ScopesStack = Vec<HashMap<String, bool>>;

enum StmtOrExpr {
    S(RcStmt),
    E(RcExpr),
}

macro_rules! to_expr {
    ($op:ident) => {
        StmtOrExpr::E(Rc::clone($op))
    };
}
macro_rules! to_stmt {
    ($op:ident) => {
        StmtOrExpr::S(Rc::clone($op))
    };
}

#[derive(Clone)]
enum FunctionType {
    NONE,
    FUNCTION,
}

pub struct Resolver {
    interpreter: SharedInterpreter,
    scopes: ScopesStack,
    current_function: FunctionType,
}

type ResolverResult = Result<(), LoxError<&'static str>>;

impl Resolver {
    pub fn new(interpreter: &SharedInterpreter) -> Self {
        Resolver {
            interpreter: Rc::clone(interpreter),
            scopes: Vec::new(),
            current_function: FunctionType::NONE,
        }
    }
    pub fn resolve_statements(&mut self, stmts: &Vec<RcStmt>) -> ResolverResult {
        for st in stmts.iter() {
            self.resolve(to_stmt!(st))?;
        }
        Ok(())
    }
    fn resolve(&mut self, stmt_or_expr: StmtOrExpr) -> ResolverResult {
        match stmt_or_expr {
            StmtOrExpr::S(stmt) => match &*stmt {
                Stmt::Block { statements } => {
                    self.begin_scope();
                    self.resolve_statements(statements)?;
                    self.end_scope();
                    Ok(())
                }
                Stmt::Expression { expr } => self.resolve(to_expr!(expr)),
                Stmt::Var { name, initializer } => {
                    self.declare(name)?;
                    if let Some(i) = initializer {
                        self.resolve(to_expr!(i))?;
                    }
                    self.define(name);
                    Ok(())
                }
                Stmt::Function { name, params, body } => {
                    self.declare(name)?;
                    self.define(name);

                    self.resolve_function(params, body, FunctionType::FUNCTION)?;
                    Ok(())
                }
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.resolve(to_expr!(condition))?;
                    self.resolve(to_stmt!(then_branch))?;
                    if let Some(el) = else_branch {
                        self.resolve(to_stmt!(el))?;
                    }
                    Ok(())
                }
                Stmt::Print { expr } => self.resolve(to_expr!(expr)),
                Stmt::Return { keyword, value } => {
                    if matches!(self.current_function, FunctionType::NONE) {
                        return Err(LoxError::ParseError {
                            token: Rc::clone(keyword),
                            message: "Can't return from top-level code.",
                        });
                    }

                    let ex = Rc::clone(value);
                    match &*ex {
                        Expr::Literal(Literal::NIL) => {}
                        _ => {
                            self.resolve(to_expr!(value))?;
                        }
                    }
                    Ok(())
                }
                Stmt::While { condition, body } => {
                    self.resolve(to_expr!(condition))?;
                    self.resolve(to_stmt!(body))
                }
            },
            StmtOrExpr::E(expr) => match &*expr {
                Expr::Variable { name } => {
                    if !self.scopes.is_empty() {
                        if let Some(v) = self.scopes.last().unwrap().get(&name.lexeme) {
                            if !v {
                                return Err(LoxError::ParseError {
                                    token: Rc::clone(name),
                                    message: "Can't read local variable in its own initializer.",
                                });
                            }
                        }
                        self.resolve_local(&expr, Rc::clone(name));
                    }
                    Ok(())
                }
                Expr::Assign { name, value } => {
                    self.resolve(StmtOrExpr::E(Rc::clone(value)))?;
                    self.resolve_local(&expr, Rc::clone(name));
                    Ok(())
                }
                Expr::Binary {
                    left,
                    operator: _,
                    right,
                } => {
                    self.resolve(to_expr!(left))?;
                    self.resolve(to_expr!(right))
                }
                Expr::Grouping(e) => self.resolve(to_expr!(e)),
                Expr::Literal(_e) => Ok(()),
                Expr::Logical {
                    left,
                    operator: _,
                    right,
                } => {
                    self.resolve(to_expr!(left))?;
                    self.resolve(to_expr!(right))
                }
                Expr::Unary { operator: _, right } => self.resolve(to_expr!(right)),
                _ => Ok(()),
            },
        }
    }
    fn resolve_local(&mut self, expr: &RcExpr, name: RcToken) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            println!("scope: {:?}, name: {}", scope, &name.lexeme);
            if scope.contains_key(&name.lexeme) {
                println!("contains!");
                self.interpreter.borrow_mut().resolve(expr, depth);
                return;
            }
        }
    }
    fn resolve_function(
        &mut self,
        params: &Vec<RcToken>,
        body: &Rc<Vec<RcStmt>>,
        func_type: FunctionType,
    ) -> ResolverResult {
        let enclosing_function = self.current_function.clone();
        self.current_function = func_type;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
        println!("{:?}", self.scopes)
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare(&mut self, name: &RcToken) -> ResolverResult {
        if self.scopes.is_empty() {
            return Ok(());
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            return Err(LoxError::ParseError {
                token: Rc::clone(name),
                message: "Already a variable with this name in this scope.",
            });
        }
        scope.insert(name.lexeme.clone(), false);
        Ok(())
    }
    fn define(&mut self, name: &RcToken) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        *scope.get_mut(&name.lexeme).unwrap() = true;
    }
}
