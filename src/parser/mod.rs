#![allow(dead_code)]

mod ast;

use ast::AstNode;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

pub struct ASTBuilder {}

impl ASTBuilder {
    fn build_ast(pair: pest::iterators::Pair<Rule>) -> Result<Box<AstNode>, String> {
        match pair.as_rule() {
            Rule::CompUnit => {
                let children = pair
                    .into_inner()
                    .map(ASTBuilder::build_ast)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Box::new(AstNode::CompUnit(children)))
            }

            _ => unimplemented!(),
        }
    }

    fn display_ast_inner(pair: pest::iterators::Pair<Rule>, prefix: String, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{:?}", prefix, connector, pair.as_rule());

        let mut inner = pair.into_inner().peekable();
        while let Some(p) = inner.next() {
            let is_last_child = inner.peek().is_none();
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            ASTBuilder::display_ast_inner(p, new_prefix, is_last_child);
        }
    }

    fn display_ast(pair: pest::iterators::Pair<Rule>) {
        println!("AST");
        ASTBuilder::display_ast_inner(pair, "".to_string(), true);
    }
}

pub fn parse(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::parse, input).map_err(|e| format!("{}", e))?;
    ASTBuilder::display_ast(parse_result.into_iter().next().unwrap());
    Ok(())
}

#[test]
fn test_parser() {
    let src = std::fs::read_to_string("./tests/parser/sample2.in").unwrap_or_default();
    parse(&src).unwrap_or_else(|e| {
        println!("{}", e);
        panic!()
    });
}
