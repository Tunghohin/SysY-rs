pub mod token;

use anyhow::Result;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./lexer/SysY.pest"]
pub(crate) struct SysYParser;

pub fn tokenize(input: &str) -> Result<Vec<token::Token>, String> {
    SysYParser::parse(Rule::program, input)
        .map_err(|e| match e.line_col {
            LineColLocation::Pos((line, _col)) => {
                let literal = matches!(e.variant, ErrorVariant::ParsingError { .. })
                    .then(|| match e.location {
                        InputLocation::Pos(pos) => input.get(pos..pos + 1).unwrap_or(""),
                        InputLocation::Span((start, end)) => input.get(start..end).unwrap_or(""),
                    })
                    .unwrap_or("");
                format!(
                    "Error type A at Line {}: Mysterious character \"{}\".",
                    line, literal
                )
            }
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
                let literal = matches!(e.variant, ErrorVariant::ParsingError { .. })
                    .then(|| match e.location {
                        InputLocation::Pos(pos) => input.get(pos..pos + 1).unwrap_or(""),
                        InputLocation::Span((start, end)) => input.get(start..end).unwrap_or(""),
                    })
                    .unwrap_or("");
                format!(
                    "Error type B from line {}, column {} to line {}, column {}, near '{}'",
                    start_line, start_col, end_line, end_col, literal
                )
            }
        })
        .and_then(|mut pairs| {
            pairs
                .next()
                .ok_or_else(|| "No input".to_string())
                .map(|pair| pair.into_inner())
        })
        .map(|inner_pairs| {
            inner_pairs
                .map(|pair| {
                    let (line, col) = pair.as_span().start_pos().line_col();
                    let lexeme = pair.as_str();
                    let kind = match pair.as_rule() {
                        // Keywords
                        Rule::CONST => token::TokenKind::Const,
                        Rule::INT => token::TokenKind::Int,
                        Rule::VOID => token::TokenKind::Void,
                        Rule::IF => token::TokenKind::If,
                        Rule::ELSE => token::TokenKind::Else,
                        Rule::WHILE => token::TokenKind::While,
                        Rule::BREAK => token::TokenKind::Break,
                        Rule::CONTINUE => token::TokenKind::Continue,
                        Rule::RETURN => token::TokenKind::Return,

                        // Identifiers and literals
                        Rule::IDENT => token::TokenKind::Ident,
                        Rule::INTEGER_CONST => token::TokenKind::IntegerConst,

                        // Operators
                        Rule::PLUS => token::TokenKind::Plus,
                        Rule::MINUS => token::TokenKind::Minus,
                        Rule::MUL => token::TokenKind::Mul,
                        Rule::DIV => token::TokenKind::Div,
                        Rule::MOD => token::TokenKind::Mod,
                        Rule::ASSIGN => token::TokenKind::Assign,
                        Rule::EQ => token::TokenKind::Eq,
                        Rule::NEQ => token::TokenKind::Neq,
                        Rule::LT => token::TokenKind::Lt,
                        Rule::GT => token::TokenKind::Gt,
                        Rule::LE => token::TokenKind::Le,
                        Rule::GE => token::TokenKind::Ge,
                        Rule::NOT => token::TokenKind::Not,
                        Rule::AND => token::TokenKind::And,
                        Rule::OR => token::TokenKind::Or,

                        // Delimiters / punctuation
                        Rule::L_PAREN => token::TokenKind::LParen,
                        Rule::R_PAREN => token::TokenKind::RParen,
                        Rule::L_BRACE => token::TokenKind::LBrace,
                        Rule::R_BRACE => token::TokenKind::RBrace,
                        Rule::L_BRACKT => token::TokenKind::LBrackt,
                        Rule::R_BRACKT => token::TokenKind::RBrackt,
                        Rule::COMMA => token::TokenKind::Comma,
                        Rule::SEMICOLON => token::TokenKind::Semicolon,

                        // Comments
                        Rule::LINE_COMMENT => token::TokenKind::LineComment,
                        Rule::MULTILINE_COMMENT => token::TokenKind::MultilineComment,

                        // Whitespace
                        Rule::WHITESPACE => token::TokenKind::Whitespace,
                        Rule::NEWLINE => token::TokenKind::Newline,

                        // Eof
                        Rule::EOI => token::TokenKind::Eof,

                        // Default
                        _ => token::TokenKind::Unknown,
                    };

                    token::Token {
                        kind,
                        lexeme,
                        len: lexeme.len(),
                        pos: (line, col),
                    }
                })
                .collect()
        })
}
