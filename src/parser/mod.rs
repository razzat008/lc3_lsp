pub mod tokenizer;
pub mod ast;
use std::fmt::{Display, Write};

#[derive(Debug)]
pub(crate) enum Token {
    ADD,
    AND,
    BR,
    JMP,
    JSR,
    JSRR,
    LD,
    LDI,
    LDR,
    LEA,
    NOT,
    RET,
    RTI,
    ST,
    STI,
    STR,
    TRAP,
    EOF,
    Value(u16),
    WhiteSpace(WhiteSpace),
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ADD => write!(f, "ADD"),
            Token::AND => write!(f, "AND"),
            Token::BR => write!(f, "BR"),
            Token::JMP => write!(f, "JMP"),
            Token::JSR => write!(f, "JSR"),
            Token::JSRR => write!(f, "JSRR"),
            Token::LD => write!(f, "LD"),
            Token::LDI => write!(f, "LDI"),
            Token::LDR => write!(f, "LDR"),
            Token::LEA => write!(f, "LEA"),
            Token::NOT => write!(f, "NOT"),
            Token::RET => write!(f, "RET"),
            Token::RTI => write!(f, "RTI"),
            Token::ST => write!(f, "ST"),
            Token::STI => write!(f, "STI"),
            Token::STR => write!(f, "STR"),
            Token::TRAP => write!(f, "TRAP"),
            Token::EOF => write!(f, "EOF"),
            Token::Value(val) => write!(f, "{val}"),
            Token::WhiteSpace(ws) => write!(f, "{ws}"),
        }
    }
}

#[derive(Debug)]
pub enum WhiteSpace {
    SPACE,
    TAB,
    NEWLINE,
}

impl Display for WhiteSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhiteSpace::SPACE => write!(f, " "),
            WhiteSpace::TAB => write!(f, "\t"),
            WhiteSpace::NEWLINE => write!(f, "\n"),
        }
    }
}
