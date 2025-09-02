pub mod token;

use anyhow::Result;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;
use std::{fs, slice::RSplit};

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

pub fn tokenize(input: &str) -> Result<Vec<token::Token>, Vec<String>> {
    let mut pos_offset: usize = 0;
    let mut line_offset = 0;
    let mut errors = Vec::new();
    let mut tokens = Vec::new();
    while pos_offset < input.len() {
        let remaining = &input[pos_offset..];
        match SysYParser::parse(Rule::tokenize, remaining) {
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
