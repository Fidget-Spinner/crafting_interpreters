use crate::expr::*;
use crate::lox::{Lox, LoxError};
use crate::stmt::Stmt;
use crate::token::*;
use crate::token_type::TokenType::*;
use std::fmt::Display;

pub struct Parser<'a> {
    lox: &'a mut Lox,
    tokens: Vec<Box<Token>>,
    current: usize,
}

type ExprResult = Result<Expr, LoxError<String>>;

type StmtResult = Result<Stmt, LoxError<String>>;

macro_rules! check {
    ($self:ident, $types:pat) => {
        if $self.is_at_end() {
            false
        } else {
            matches!(&$self.peek().type_, $types)
        }
    };
}

macro_rules! match_ {
    ($self:ident, $types:pat) => {
        if check!($self, $types) {
            $self.advance();
            true
        } else {
            false
        }
    };
}

macro_rules! consume {
    ($self:ident, $type_:pat, $message:expr) => {
        if check!($self, $type_) {
            Ok($self.advance())
        } else {
            Err(Parser::error($self.peek(), String::from($message)))
        }
    };
    ($self:ident, $type_:pat, $message:literal, $($args: tt) *) => {
        if check!($self, $type_) {
            Ok($self.advance())
        } else {
            Err(Parser::error($self.peek(), format!($message, $($args,) *)))
        }
    };
}

#[allow(dead_code)]
impl Parser<'_> {
    pub fn new(lox: &mut Lox, tokens: Vec<Box<Token>>) -> Parser {
        Parser {
            lox,
            tokens,
            current: 0,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>, LoxError<String>> {
        let mut statements: Vec<Box<Stmt>> = Vec::new();
        while !self.is_at_end() {
            statements.push(Box::new(self.declaration()?));
        }
        Ok(statements)
    }
    fn expression(&mut self) -> ExprResult {
        self.assignment()
    }
    fn declaration(&mut self) -> StmtResult {
        let res = if match_!(self, FUN) {
            self.function("function")
        } else if match_!(self, VAR) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match res {
            Err(res) => {
                self.synchronize();
                Err(res)
            }
            _ => res,
        }
    }
    fn statement(&mut self) -> StmtResult {
        if match_!(self, FOR) {
            return self.for_statement();
        }
        if match_!(self, IF) {
            return self.if_statement();
        }
        if match_!(self, PRINT) {
            return self.print_statement();
        }
        if match_!(self, RETURN) {
            return self.return_statement();
        }
        if match_!(self, WHILE) {
            return self.while_statement();
        }
        if match_!(self, LEFT_BRACE) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        self.expression_statement()
    }
    fn for_statement(&mut self) -> StmtResult {
        consume!(self, LEFT_PAREN, "Expect '(' after 'for'.")?;

        let initializer = if match_!(self, SEMICOLON) {
            None
        } else if match_!(self, VAR) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let mut condition = None;
        if !check!(self, SEMICOLON) {
            condition = Some(self.expression()?);
        }
        consume!(self, SEMICOLON, "Expect ';' after loop condition")?;

        let mut increment = None;
        if !check!(self, RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }
        consume!(self, RIGHT_PAREN, "Expect ')' after for clauses.")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    Box::new(body),
                    Box::new(Stmt::Expression {
                        expr: Box::new(increment),
                    }),
                ],
            }
        }

        if condition.is_none() {
            condition = Some(Expr::Literal(Literal::BOOL(true)));
        }
        body = Stmt::While {
            condition: Box::new(condition.unwrap()),
            body: Box::new(body),
        };
        if initializer.is_some() {
            body = Stmt::Block {
                statements: vec![Box::new(initializer.unwrap()), Box::new(body)],
            };
        }
        Ok(body)
    }
    fn if_statement(&mut self) -> StmtResult {
        consume!(self, LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        consume!(self, RIGHT_PAREN, "Expect ')' after 'if'.")?;

        let then_branch = self.statement()?;
        let else_branch = if match_!(self, ELSE) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }
    fn print_statement(&mut self) -> StmtResult {
        let value = self.expression()?;
        consume!(self, SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print {
            expr: Box::new(value),
        })
    }
    fn return_statement(&mut self) -> StmtResult {
        let keyword = self.previous();
        let value = if !check!(self, SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal(Literal::NIL)
        };

        consume!(self, SEMICOLON, "Expect ';' after return value.")?;
        Ok(Stmt::Return {
            keyword,
            value: Box::new(value),
        })
    }
    fn var_declaration(&mut self) -> StmtResult {
        let name = consume!(self, IDENTIFIER, "Expect variable name.")?;
        let mut initializer: Option<Box<Expr>> = None;
        if match_!(self, EQUAL) {
            initializer = Some(Box::new(self.expression()?));
        }
        consume!(self, SEMICOLON, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var { name, initializer })
    }
    fn while_statement(&mut self) -> StmtResult {
        consume!(self, LEFT_PAREN, "Expect '(', after 'while'.")?;
        let condition = self.expression()?;
        consume!(self, RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }
    fn expression_statement(&mut self) -> StmtResult {
        let expr = self.expression()?;
        consume!(self, SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }
    fn function(&mut self, kind: &'static str) -> StmtResult {
        let name = consume!(self, IDENTIFIER, "Expect {} name.", kind)?;
        consume!(self, LEFT_PAREN, "Expect '(' after {} name.", kind)?;
        let mut parameters: Vec<Box<Token>> = Vec::new();
        if !check!(self, RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    self.lox.error(Parser::error(
                        self.peek(),
                        "Can't have more than 255 parameters.",
                    ));
                }
                parameters.push(consume!(self, IDENTIFIER, "Expect parameter name.")?);
                if !match_!(self, COMMA) {
                    break;
                }
            }
        }
        consume!(self, RIGHT_PAREN, "Expect ')' after parameters.")?;

        consume!(self, LEFT_BRACE, "Expect '{{ before {} body.", kind)?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        })
    }
    fn block(&mut self) -> Result<Vec<Box<Stmt>>, LoxError<String>> {
        let mut statements = Vec::<Box<Stmt>>::new();
        while !check!(self, RIGHT_BRACE) && !self.is_at_end() {
            statements.push(Box::new(self.declaration()?));
        }
        consume!(self, RIGHT_BRACE, "Expect '}' after block.")?;
        Ok(statements)
    }
    fn assignment(&mut self) -> ExprResult {
        let expr = self.or()?;
        if match_!(self, EQUAL) {
            let equals = self.previous();
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                _ => self
                    .lox
                    .error(Parser::error(&equals, "Invalid assignment target.")),
            }
        }
        Ok(expr)
    }
    fn or(&mut self) -> ExprResult {
        let mut expr = self.and()?;
        while match_!(self, OR) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn and(&mut self) -> ExprResult {
        let mut expr = self.equality()?;
        while match_!(self, AND) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn equality(&mut self) -> ExprResult {
        let mut expr = self.comparison()?;
        while match_!(self, BANG_EQUAL | EQUAL_EQUAL) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn advance(&mut self) -> Box<Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn comparison(&mut self) -> ExprResult {
        let mut expr: Expr = self.term()?;
        while match_!(self, GREATER | GREATER_EQUAL | LESS | LESS_EQUAL) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn term(&mut self) -> ExprResult {
        let mut expr: Expr = self.factor()?;
        while match_!(self, MINUS | PLUS) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn factor(&mut self) -> ExprResult {
        let mut expr: Expr = self.unary()?;
        while match_!(self, SLASH | STAR) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn unary(&mut self) -> ExprResult {
        if match_!(self, BANG | MINUS) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.call()
    }
    fn finish_call(&mut self, callee: Expr) -> ExprResult {
        let mut arguments = Vec::<Box<Expr>>::new();
        if !check!(self, RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    self.lox.error(Parser::error(
                        self.peek(),
                        "Can't have more than 255 arguments",
                    ));
                }
                arguments.push(Box::new(self.expression()?));
                if !match_!(self, COMMA) {
                    break;
                }
            }
        }
        let paren = consume!(self, RIGHT_PAREN, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }
    fn call(&mut self) -> ExprResult {
        let mut expr = self.primary()?;

        loop {
            if match_!(self, LEFT_PAREN) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn primary(&mut self) -> ExprResult {
        if match_!(self, FALSE) {
            return Ok(Expr::Literal(Literal::BOOL(false)));
        }
        if match_!(self, TRUE) {
            return Ok(Expr::Literal(Literal::BOOL(true)));
        }
        if match_!(self, NIL) {
            return Ok(Expr::Literal(Literal::NIL));
        }
        if match_!(self, NUMBER | STRING) {
            return Ok(Expr::Literal(self.previous().literal));
        }
        if match_!(self, IDENTIFIER) {
            return Ok(Expr::Variable {
                name: self.previous(),
            });
        }
        if match_!(self, LEFT_PAREN) {
            let expr = self.expression()?;
            consume!(self, RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        Err(Parser::error(
            self.peek(),
            String::from("Expect expression"),
        ))
    }

    /* Non-production rule functions */
    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().type_, EOF)
    }
    #[inline(always)]
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    #[inline(always)]
    fn previous(&self) -> Box<Token> {
        self.tokens[self.current - 1].clone()
    }
    fn error<T: Display>(token: &Token, message: T) -> LoxError<T> {
        LoxError::ParseError {
            token: token.clone(),
            message,
        }
    }
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if matches!(self.previous().type_, SEMICOLON) {
                return;
            }

            match self.peek().type_ {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => return,
                _ => self.advance(),
            };
        }
    }
}
