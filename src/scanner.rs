use crate::lox::LoxError;
use crate::token::{Literal, RcToken, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::str;

trait Sub {
    fn substr(&self, start: usize, stop: usize) -> Self;
    fn char_at(&self, index: usize) -> char;
}

impl Sub for String {
    fn substr(&self, start: usize, stop: usize) -> Self {
        self.chars()
            .skip(start as usize)
            .take((start - stop) as usize)
            .collect()
    }
    fn char_at(&self, index: usize) -> char {
        self.as_bytes()[index] as char
    }
}

trait Alpha {
    fn is_ascii_identifier(&self) -> bool;
}

impl Alpha for u8 {
    fn is_ascii_identifier(&self) -> bool {
        self.is_ascii_alphanumeric() || *self == b'_'
    }
}

pub struct Scanner {
    source: Vec<u8>,
    pub tokens: Vec<RcToken>,
    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
}

macro_rules! match_ {
    ($self:ident, $expected:literal) => {
        if $self.is_at_end() {
            false
        } else {
            if !matches!($self.source[$self.current], $expected) {
                false
            } else {
                $self.current += 1;
                true
            }
        }
    };
}

impl Scanner {
    pub fn new(source: Vec<u8>) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                ("and", AND),
                ("class", CLASS),
                ("else", ELSE),
                ("false", FALSE),
                ("for", FOR),
                ("fun", FUN),
                ("if", IF),
                ("nil", NIL),
                ("or", OR),
                ("print", PRINT),
                ("return", RETURN),
                ("super", SUPER),
                ("this", THIS),
                ("true", TRUE),
                ("var", VAR),
                ("while", WHILE),
            ]),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), LoxError<&'static str>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Rc::from(Token::new(
            EOF,
            Vec::new(),
            Literal::NIL,
            self.line,
        )));
        Ok(())
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), LoxError<&'static str>> {
        let c = self.advance();
        match c {
            b'(' => self.add_token(LEFT_PAREN),
            b')' => self.add_token(RIGHT_PAREN),
            b'{' => self.add_token(LEFT_BRACE),
            b'}' => self.add_token(RIGHT_BRACE),
            b',' => self.add_token(COMMA),
            b'.' => self.add_token(DOT),
            b'-' => self.add_token(MINUS),
            b'+' => self.add_token(PLUS),
            b';' => self.add_token(SEMICOLON),
            b'*' => self.add_token(STAR),
            b'!' => {
                let matches = match_!(self, b'=');
                self.add_token(if matches { BANG_EQUAL } else { BANG })
            }
            b'=' => {
                let matches = match_!(self, b'=');
                self.add_token(if matches { EQUAL_EQUAL } else { EQUAL })
            }
            b'<' => {
                let matches = match_!(self, b'=');
                self.add_token(if matches { LESS_EQUAL } else { LESS })
            }
            b'>' => {
                let matches = match_!(self, b'=');
                self.add_token(if matches { GREATER_EQUAL } else { GREATER })
            }
            b'/' => {
                let matches = match_!(self, b'/');
                // a comment -- //
                if matches {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(SLASH);
                }
            }
            // ignore whitespace
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'"' => return self.string(),
            // numbers
            b'0'..=b'9' => self.number(),
            // identifiers (alpha)
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => self.identifier(),
            _ => {
                return Err(LoxError::ScanError {
                    line: self.line,
                    message: &"Unexpected character.",
                });
            }
        }
        Ok(())
    }
    fn advance(&mut self) -> u8 {
        let res = self.source[self.current];
        self.current += 1;
        res
    }

    fn add_token_literal(&mut self, type_: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Rc::from(Token::new(
            type_,
            text.to_vec(),
            literal,
            self.line,
        )));
    }

    fn add_token(&mut self, type_: TokenType) {
        self.add_token_literal(type_, Literal::NIL);
    }

    // fn match_(&mut self, expected: u8) -> bool {
    //     if self.is_at_end() {
    //         return false;
    //     }
    //     if self.source[self.current] != expected {
    //         return false;
    //     }
    //     self.current += 1;
    //     true
    // }

    #[inline(always)]
    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current]
    }

    fn string(&mut self) -> Result<(), LoxError<&'static str>> {
        // read till closing quote
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxError::ScanError {
                line: self.line,
                message: &"Unterminated string",
            });
        }
        // the closing "
        self.advance();
        // Trim the surrounding quotes.
        let value = str::from_utf8(&self.source[self.start + 1..self.current - 1])
            .expect("Invalid UTF8")
            .to_string();
        self.add_token_literal(STRING, Literal::STRING(value));
        Ok(())
    }

    #[inline(always)]
    fn is_digit(c: u8) -> bool {
        matches!(c, b'0'..=b'9')
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }
        // look for fractional part .
        if self.peek() == b'.' && Scanner::is_digit(self.peek_next()) {
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }
        self.add_token_literal(
            NUMBER,
            Literal::NUMBER(
                str::from_utf8(&self.source[self.start..self.current])
                    .expect("Invalid UTF8")
                    .parse()
                    .expect("Invalid float"),
            ),
        );
    }

    fn peek_next(&mut self) -> u8 {
        if self.current + 1 >= self.source.len() {
            return b'\0';
        }
        self.source[self.current + 1]
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_identifier() {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token_type = self
            .keywords
            .get(&str::from_utf8(text).expect("invalid unicode"))
            .cloned()
            .unwrap_or(IDENTIFIER);
        self.add_token(token_type);
    }
}
