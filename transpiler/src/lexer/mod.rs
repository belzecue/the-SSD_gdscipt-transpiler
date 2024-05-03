//mod chumsky;
pub mod handwritten;

use std::fmt::Display;
//pub use chumsky::*;
pub use handwritten::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Anotation(String),
    GetNode(String),
    String(String),
    RawString(String),
    Number(u64),
    FPNumber(f64),

    Op(String),
    Ctrl(String),

    Indent,
    DeIdent,
    NewLine,
    Comment(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::FPNumber(n) => write!(f, "{}", n),

            Token::Identifier(i) => write!(f, "{}", i),
            Token::Anotation(a) => write!(f, "{}", a),
            Token::GetNode(p) => write!(f, "{}", p),

            Token::String(s) => write!(f, "{}", s),
            Token::RawString(s) => write!(f, "{}", s),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),

            Token::Indent | Token::DeIdent | Token::NewLine => Ok(()),
            Token::Comment(c) => write!(f, "{}", c),
        }
    }
}
