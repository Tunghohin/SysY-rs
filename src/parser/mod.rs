mod ast;

use ast::AstNode;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

pub struct ASTBuilder {
}

impl ASTBuilder {
    fn build_ast(pair: pest::iterators::Pair<Rule>, depth: usize) {
    }

    fn display_ast(pair: pest::iterators::Pair<Rule>, depth: usize) {
        let indent = "  ".repeat(depth);
        let rule_type = pair.as_rule();
        println!("{}Rule: {:?}", indent, rule_type);
 
        pair.into_inner().for_each(|inner| {
            ASTBuilder::display_ast(inner, depth + 1);
        });
    }
}

pub fn parse(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::parse, input).map_err(|e| format!("{}", e))?;
    ASTBuilder::display_ast(parse_result.into_iter().next().unwrap(), 0);
    Ok(())
}

#[test]
fn test_parser() {
    let src = std::fs::read_to_string("./tests/parser/sample2.in").unwrap_or_default();
    parse(&src).unwrap_or_else(|e| { println!("{}", e); panic!() });
}
