#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Unknown,
    Eof,

    // Keywords
    Const,
    Int,
    Void,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,

    // Identifiers and literals
    Ident,
    IntegerConst,

    // Operators
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Assign,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Not,
    And,
    Or,

    // Delimiters / punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrackt,
    RBrackt,
    Comma,
    Semicolon,

    // Comments
    LineComment,
    MultilineComment,

    // Whitespace / newline (optional, depends if you want to track it)
    Whitespace,
    Newline,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub len: usize,
    pub pos: (usize, usize),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenKind::Unknown => "Unknown",
            TokenKind::Eof => "EOF",

            TokenKind::Const => "CONST",
            TokenKind::Int => "INT",
            TokenKind::Void => "VOID",
            TokenKind::If => "IF",
            TokenKind::Else => "ELSE",
            TokenKind::While => "WHILE",
            TokenKind::Break => "BREAK",
            TokenKind::Continue => "CONTINUE",
            TokenKind::Return => "RETURN",

            TokenKind::Ident => "IDENT",
            TokenKind::IntegerConst => "INTEGER_CONST",

            TokenKind::Plus => "PLUS",
            TokenKind::Minus => "MINUS",
            TokenKind::Mul => "MUL",
            TokenKind::Div => "DIV",
            TokenKind::Mod => "MOD",
            TokenKind::Assign => "ASSIGN",
            TokenKind::Eq => "EQ",
            TokenKind::Neq => "NEQ",
            TokenKind::Lt => "LT",
            TokenKind::Gt => "GT",
            TokenKind::Le => "LE",
            TokenKind::Ge => "GE",
            TokenKind::Not => "NOT",
            TokenKind::And => "AND",
            TokenKind::Or => "OR",

            TokenKind::LParen => "L_PAREN",
            TokenKind::RParen => "R_PAREN",
            TokenKind::LBrace => "L_BRACE",
            TokenKind::RBrace => "R_BRACE",
            TokenKind::LBrackt => "L_BRACKT",
            TokenKind::RBrackt => "R_BRACKT",
            TokenKind::Comma => "COMMA",
            TokenKind::Semicolon => "SEMICOLON",

            TokenKind::LineComment => "LINE_COMMENT",
            TokenKind::MultilineComment => "MULTILINE_COMMENT",

            TokenKind::Whitespace => "WHITESPACE",
            TokenKind::Newline => "NEWLINE",
        };
        write!(f, "{}", name)
    }
}

fn parse_integer_const(s: &str) -> Result<i64, String> {
    if s.starts_with("0x") || s.starts_with("0X") {
        i64::from_str_radix(&s[2..], 16).map_err(|_| format!("NaN: {}", s))
    } else if s.starts_with('0') && s.len() > 1 {
        if s[1..].chars().all(|c| c >= '0' && c <= '7') {
            i64::from_str_radix(&s[1..], 8).map_err(|_| format!("NaN: {}", s))
        } else {
            Err(format!("NaN: {}", s))
        }
    } else {
        s.parse::<i64>().map_err(|_| format!("NaN: {}", s))
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::IntegerConst => {
                let decimal = parse_integer_const(self.lexeme).map_err(|_| fmt::Error)?;

                write!(f, "{} {} at Line {}.", self.kind, decimal, self.pos.0)
            }
            _ => {
                write!(f, "{} {} at Line {}.", self.kind, self.lexeme, self.pos.0)
            }
        }
    }
}
