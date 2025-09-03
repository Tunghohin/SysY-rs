pub mod token;

use anyhow::Result;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;
use std::{fs, slice::RSplit};

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYLexer;

pub fn tokenize(input: &str) -> Result<Vec<token::Token>, Vec<String>> {
    let mut pos_offset: usize = 0;
    let mut line_offset = 0;
    let mut errors = Vec::new();
    let mut tokens = Vec::new();
    while pos_offset < input.len() {
        let remaining = &input[pos_offset..];
        match SysYLexer::parse(Rule::tokenize, remaining) {
            Ok(mut pairs) => {
                pairs
                    .next()
                    .ok_or_else(|| vec!["No input".to_string()])?
                    .into_inner()
                    .for_each(|pair| {
                        let (line, col) = pair.as_span().start_pos().line_col();
                        let lexeme = pair.as_str();
                        let kind = match pair.as_rule() {
                            // Keywords
                            Rule::Const => token::TokenKind::Const,
                            Rule::Int => token::TokenKind::Int,
                            Rule::Void => token::TokenKind::Void,
                            Rule::If => token::TokenKind::If,
                            Rule::Else => token::TokenKind::Else,
                            Rule::While => token::TokenKind::While,
                            Rule::Break => token::TokenKind::Break,
                            Rule::Continue => token::TokenKind::Continue,
                            Rule::Return => token::TokenKind::Return,
                            
                            // Identifiers and literals
                            Rule::Ident => token::TokenKind::Ident,
                            Rule::IntegerConst => token::TokenKind::IntegerConst,
                            
                            // Operators
                            Rule::Plus => token::TokenKind::Plus,
                            Rule::Minus => token::TokenKind::Minus,
                            Rule::Mul => token::TokenKind::Mul,
                            Rule::Div => token::TokenKind::Div,
                            Rule::Mod => token::TokenKind::Mod,
                            Rule::Assign => token::TokenKind::Assign,
                            Rule::Eq => token::TokenKind::Eq,
                            Rule::Neq => token::TokenKind::Neq,
                            Rule::Lt => token::TokenKind::Lt,
                            Rule::Gt => token::TokenKind::Gt,
                            Rule::Le => token::TokenKind::Le,
                            Rule::Ge => token::TokenKind::Ge,
                            Rule::Not => token::TokenKind::Not,
                            Rule::And => token::TokenKind::And,
                            Rule::Or => token::TokenKind::Or,
                            
                            // Delimiters / punctuation
                            Rule::LParen => token::TokenKind::LParen,
                            Rule::RParen => token::TokenKind::RParen,
                            Rule::LBrace => token::TokenKind::LBrace,
                            Rule::RBrace => token::TokenKind::RBrace,
                            Rule::LBracket => token::TokenKind::LBracket,
                            Rule::RBracket => token::TokenKind::RBracket,
                            Rule::Comma => token::TokenKind::Comma,
                            Rule::Semicolon => token::TokenKind::Semicolon,
                            
                            // Eof
                            Rule::EOI => token::TokenKind::Eof,
                            
                            // Default
                            _ => token::TokenKind::Unknown,                            
                        };
                        tokens.push(token::Token {
                            kind,
                            lexeme,
                            len: lexeme.len(),
                            pos: (line, col),
                        });
                    });
                break;
            }
            Err(e) => {
                match e.line_col {
                    LineColLocation::Pos((line, _col)) => {
                        let literal = matches!(e.variant, ErrorVariant::ParsingError { .. })
                            .then(|| match e.location {
                                InputLocation::Pos(pos) => remaining.get(pos..pos + 1).unwrap_or(""),
                                InputLocation::Span((start, end)) => {
                                    remaining.get(start..end).unwrap_or("")
                                }
                            })
                            .unwrap_or("");
                        errors.push(format!(
                            "Error type A at Line {}: Mysterious character \"{}\".",
                            line + line_offset,
                            literal
                        ));
                        line_offset += line - 1;
                    }
                    LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
                        let literal = matches!(e.variant, ErrorVariant::ParsingError { .. })
                            .then(|| match e.location {
                                InputLocation::Pos(pos) => remaining.get(pos..pos + 1).unwrap_or(""),
                                InputLocation::Span((start, end)) => {
                                    remaining.get(start..end).unwrap_or("")
                                }
                            })
                            .unwrap_or("");
                        errors.push(format!(
                            "Error type B from line {}, column {} to line {}, column {}, near '{}'",
                            start_line,
                            start_col,
                            end_line + line_offset,
                            end_col,
                            literal
                        ));
                        line_offset += end_line - 1;
                    }
                }
                match e.location {
                    InputLocation::Pos(error_pos) => {
                        pos_offset += error_pos + 1;
                    }
                    InputLocation::Span((_start, end)) => {
                        pos_offset += end;
                    }
                }
                if pos_offset >= input.len() {
                    break;
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

#[test]
fn test_lexer() {
    let case_dir = std::path::Path::new("./tests/lexer");

    let mut entries: Vec<_> = fs::read_dir(case_dir)
        .unwrap_or_else(|_| panic!("Failed to read dir"))
        .map(|res| res.unwrap().path())
        .filter(|path| path.extension().map(|e| e == "in").unwrap_or(false))
        .collect();

    entries.sort();

    for entry in entries {
        let mut out_buffer = String::new();
        let input_path = entry.to_str().unwrap();
        let solution_path = input_path.trim_end_matches(".in").to_string() + ".out";


        let input = fs::read_to_string(input_path).expect("Failed to read file");
        tokenize(&input)
            .unwrap_or_else(|errs| {
                errs.iter().for_each(|err| {
                    out_buffer.push_str(&format!("{}\n", err));
                });
                vec![]
            })
            .iter()
            .for_each(|token| match token.kind {
                token::TokenKind::Eof => {}
                _ => {
                    out_buffer.push_str(&format!("{}\n", token));
                }
            });

        let solution = fs::read_to_string(solution_path).unwrap_or_else(|_| {
            panic!("Failed to read solution");
        });
        if out_buffer != solution {
            panic!("Test failed for {}, expected: \n{}, found: \n{}", input_path, solution, out_buffer);
        }
        println!("{} passed.", input_path.rsplit('/').next().unwrap().trim_end_matches(".in"));
    }
}
