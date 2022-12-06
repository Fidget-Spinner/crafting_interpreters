use super::token_type::TokenType;
use std::hash::{Hash, Hasher};
use std::mem;
use std::rc::Rc;
use std::str;

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

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::IDENTIFIER(s) => s.hash(state),
            Literal::STRING(s) => s.hash(state),
            Literal::BOOL(b) => b.hash(state),
            Literal::NIL => state.write_u8(1),
            Literal::NUMBER(f) => integer_decode(f.clone()).hash(state),
        }
    }
}

impl Eq for Literal {}

/* Code from https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust */
fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
