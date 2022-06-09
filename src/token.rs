use super::token_type::TokenType;
#[allow(unused_imports)]
use std::any::Any;
#[allow(unused_imports)]
use std::str;
use std::rc::Rc;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    IDENTIFIER(String),
    STRING(String),
    NUMBER(f64),
    BOOL(bool),
    NIL,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

#[allow(dead_code)]
impl Token {
    pub fn new(type_: TokenType, lexeme: Vec<u8>, literal: Literal, line: usize) -> Token {
        let s = str::from_utf8(&lexeme).expect("Invalid UTF8").to_string();
        Token {
            type_,
            lexeme: s,
            literal,
            line,
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "[Token] type: {:?}, lexeme: {}, literal: {:?}, line: {}",
            self.type_, self.lexeme, self.literal, self.line
        )
    }
}

pub type RcToken = Rc<Token>;

impl Literal {
    pub fn to_string(&self) -> String {
        match self {
            Literal::IDENTIFIER(id) => id.to_owned(),
            Literal::STRING(st) => st.to_owned(), // format!("\"{}\"", st.to_owned()),
            Literal::NUMBER(num) => {
                let mut text = format!("{}", num);
                if text.ends_with(".0") {
                    text = String::from(text.strip_suffix(".0").unwrap())
                }
                text
            }
            Literal::BOOL(bl) => format!("{}", bl),
            Literal::NIL => String::from("nil"),
        }
    }
}
