#![allow(dead_code)]

mod ast;

use ast::AstNode;
use pest::Parser;
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
            Rule::Decl => {
                let child = ASTBuilder::build_ast(pair.into_inner().next().unwrap())?;
                Ok(Box::new(AstNode::Decl(child)))
            }
            Rule::BType => {
                let btype = pair.as_str().to_string();
                Ok(Box::new(AstNode::BType(btype)))
            }
            Rule::ConstDecl => {
                let mut inner = pair.into_inner();
                inner.next(); // consume 'Const'
                let btype = ASTBuilder::build_ast(inner.next().unwrap())?;
                let const_defs = inner
                    .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
                    .map(ASTBuilder::build_ast)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Box::new(AstNode::ConstDecl { btype, const_defs }))
            }
            Rule::ConstDef => {
                let mut inner = pair.into_inner();
                let ident = inner.next().unwrap().as_str().to_string();
                let mut dimensions = Vec::new();

                while let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::LBracket {
                        inner.next(); // consume '['
                        dimensions.push(ASTBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                        inner.next(); // consume ']'
                    } else if next_pair.as_rule() == Rule::Assign {
                        inner.next(); // consume '='
                        break;
                    } else {
                        break;
                    }
                }
                let init_val = ASTBuilder::build_ast(inner.next().unwrap())?;
                Ok(Box::new(AstNode::ConstDef {
                    ident,
                    dimensions,
                    init_val,
                }))
            }
            Rule::ConstInitVal => {
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();
                if first.as_rule() == Rule::LBrace {
                    inner.next(); // consume '{'
                    let mut init_vals = Vec::new();
                    while let Some(next_pair) = inner.peek() {
                        if next_pair.as_rule() == Rule::RBrace {
                            break;
                        }
                        if next_pair.as_rule() != Rule::Comma {
                            init_vals.push(ASTBuilder::build_ast(inner.next().unwrap())?);
                        } else {
                            inner.next(); // consume ';'
                        }
                    }
                    Ok(Box::new(AstNode::ConstInitVal(
                        ast::ConstInitValType::List(init_vals),
                    )))
                } else {
                    let exp = ASTBuilder::build_ast(inner.next().unwrap())?;
                    Ok(Box::new(AstNode::ConstInitVal(ast::ConstInitValType::Exp(
                        exp,
                    ))))
                }
            }
            Rule::VarDecl => {
                let mut inner = pair.into_inner();
                let btype = ASTBuilder::build_ast(inner.next().unwrap())?;
                let var_defs = inner
                    .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
                    .map(ASTBuilder::build_ast)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Box::new(AstNode::VarDecl { btype, var_defs }))
            }
            Rule::VarDef => {
                let mut inner = pair.into_inner();
                let ident = inner.next().unwrap().as_str().to_string();
                let mut dimensions = Vec::new();
                let mut init_val = None;

                while let Some(next_pair) = inner.peek() {
                    match next_pair.as_rule() {
                        Rule::LBracket => {
                            inner.next(); // consume '['
                            dimensions.push(ASTBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                            inner.next(); // consume ']'
                        }
                        Rule::Assign => {
                            inner.next(); // consume '='
                            init_val = Some(ASTBuilder::build_ast(inner.next().unwrap())?);
                            break;
                        }
                        _ => break,
                    }
                }

                Ok(Box::new(AstNode::VarDef {
                    ident,
                    dimensions,
                    init_val,
                }))
            }
            Rule::InitVal => {
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();
                if first.as_rule() == Rule::LBrace {
                    inner.next(); // consume '{'
                    let mut init_vals = Vec::new();

                    while let Some(next_pair) = inner.peek() {
                        if next_pair.as_rule() == Rule::RBrace {
                            break;
                        }
                        if next_pair.as_rule() != Rule::Comma {
                            init_vals.push(ASTBuilder::build_ast(inner.next().unwrap())?);
                        } else if next_pair.as_rule() == Rule::Comma {
                            inner.next(); // consume ';'
                        } else {
                            unreachable!()
                        }
                    }
                    Ok(Box::new(AstNode::InitVal(ast::InitValType::List(
                        init_vals,
                    ))))
                } else {
                    let exp = ASTBuilder::build_ast(inner.next().unwrap())?;
                    Ok(Box::new(AstNode::InitVal(ast::InitValType::Exp(exp))))
                }
            }
            Rule::FuncDef => {
                let mut inner = pair.into_inner();
                let func_type = ASTBuilder::build_ast(inner.next().unwrap())?;
                let ident = inner.next().unwrap().as_str().to_string();
                inner.next(); // consume '('
                let mut params = None;
                if let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::FuncFParams {
                        params = Some(ASTBuilder::build_ast(inner.next().unwrap())?);
                    }
                }
                inner.next(); // consume ')'
                let body = ASTBuilder::build_ast(inner.next().unwrap())?;
                Ok(Box::new(AstNode::FuncDef {
                    func_type,
                    ident,
                    params,
                    body,
                }))
            }
            Rule::FuncType => {
                let func_type = pair.as_str().to_string();
                Ok(Box::new(AstNode::FuncType(func_type)))
            }
            Rule::FuncFParams => {
                let params = pair
                    .into_inner()
                    .filter(|p| p.as_rule() != Rule::Comma)
                    .map(ASTBuilder::build_ast)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Box::new(AstNode::FuncFParams(params)))
            }
            Rule::FuncFParam => {
                let mut inner = pair.into_inner();
                let btype = ASTBuilder::build_ast(inner.next().unwrap())?;
                let ident = inner.next().unwrap().as_str().to_string();
                let mut is_array = false;
                let mut dimensions = Vec::new();
                if let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::LBracket {
                        inner.next(); // consume '['
                        inner.next(); // consume ']'
                        is_array = true;
                        while let Some(bracket) = inner.peek() {
                            if bracket.as_rule() == Rule::LBracket {
                                inner.next(); // consume '['
                                dimensions.push(ASTBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                                inner.next(); // consume ']'
                            } else {
                                break;
                            }
                        }
                    }
                }

                Ok(Box::new(AstNode::FuncFParam {
                    btype,
                    ident,
                    is_array,
                    dimensions,
                }))
            }
            Rule::Block => {
                let items = pair
                    .into_inner()
                    .filter(|p| p.as_rule() != Rule::LBrace && p.as_rule() != Rule::RBrace)
                    .map(ASTBuilder::build_ast)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Box::new(AstNode::Block(items)))
            }
            Rule::BlockItem => Ok(Box::new(AstNode::BlockItem(ASTBuilder::build_ast(
                pair.into_inner().next().unwrap(),
            )?))),

            Rule::Ident
            | Rule::Plus
            | Rule::Minus
            | Rule::Mul
            | Rule::Div
            | Rule::Mod
            | Rule::Assign
            | Rule::Eq
            | Rule::Neq
            | Rule::Lt
            | Rule::Gt
            | Rule::Le
            | Rule::Ge
            | Rule::Not
            | Rule::And
            | Rule::Or
            | Rule::LParen
            | Rule::RParen
            | Rule::LBrace
            | Rule::RBrace
            | Rule::LBracket
            | Rule::RBracket
            | Rule::Comma
            | Rule::Semicolon
            | Rule::Const
            | Rule::Int
            | Rule::Void
            | Rule::If
            | Rule::Else
            | Rule::While
            | Rule::Break
            | Rule::Continue
            | Rule::Return => Err(format!("Unexpected terminal rule: {:?}", pair.as_rule())),

            _ => Err(format!("Unhandled rule: {:?}", pair.as_rule())),
        }
    }

    fn display_ast_inner(pair: pest::iterators::Pair<Rule>, prefix: String, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        if pair.as_rule() == Rule::Ident || pair.as_rule() == Rule::IntegerConst {
            let lexeme = pair.as_str();
            println!(
                "{}{}{:?} <row: {}, col: {}> {}",
                prefix,
                connector,
                pair.as_rule(),
                pair.line_col().0,
                pair.line_col().1,
                lexeme
            );
        } else {
            println!("{}{}{:?}", prefix, connector, pair.as_rule());
        }

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
        ASTBuilder::display_ast_inner(pair, "".to_string(), true);
    }
}

pub fn display_ast(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::parse, input).map_err(|e| format!("{}", e))?;
    ASTBuilder::display_ast(parse_result.into_iter().next().unwrap());
    Ok(())
}

pub fn parse(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::parse, input).map_err(|e| format!("{}", e))?;
    let _ = ASTBuilder::build_ast(parse_result.into_iter().next().unwrap());
    Ok(())
}

#[test]
fn test_parser() {
    let src = std::fs::read_to_string("./tests/parser/expr1.in").unwrap_or_default();
    display_ast(&src).unwrap_or_else(|e| {
        println!("{}", e);
        panic!()
    });
}
