#![allow(dead_code)]

mod ast;

use ast::AstNode;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

pub struct AstBuilder {}

impl AstBuilder {
    fn build_ast(pair: pest::iterators::Pair<Rule>) -> Result<Box<AstNode>, String> {
        match pair.as_rule() {
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

            _ => Ok(Box::new(AstNode::try_from(pair)?)),
        }
    }

    fn build_comp_unit(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::CompUnit(
            pair.into_inner()
                .map(AstBuilder::build_ast)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn build_decl(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::Decl(AstBuilder::build_ast(
            pair.into_inner().next().unwrap(),
        )?))
    }

    fn build_btype(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::BType(pair.as_str().to_string()))
    }

    fn build_const_decl(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();

        inner.next(); // consume 'Const'

        let btype = AstBuilder::build_ast(inner.next().unwrap())?;

        let const_defs = inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(AstBuilder::build_ast)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AstNode::ConstDecl { btype, const_defs })
    }

    fn build_const_def(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap().as_str().to_string();

        let mut dimensions = Vec::new();
        while let Some(next_pair) = inner.peek() {
            if next_pair.as_rule() == Rule::LBracket {
                inner.next(); // consume '['
                dimensions.push(AstBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                inner.next(); // consume ']'
            } else {
                break;
            }
        }

        inner.next(); // consume '='
        let init_val = AstBuilder::build_ast(inner.next().unwrap())?;

        Ok(AstNode::ConstDef {
            ident,
            dimensions,
            init_val,
        })
    }

    fn build_const_init_val(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();

        if inner.peek().unwrap().as_rule() == Rule::LBrace {
            inner.next(); // consume '{'
            let mut init_vals = Vec::new();
            while let Some(next_pair) = inner.peek() {
                if next_pair.as_rule() == Rule::RBrace {
                    break;
                }
                if next_pair.as_rule() != Rule::Comma {
                    init_vals.push(AstBuilder::build_ast(inner.next().unwrap())?);
                } else if next_pair.as_rule() == Rule::Comma {
                    inner.next(); // consume ','
                } else {
                    return Err(format!(
                        "Unexpected rule in init_val: {:?}",
                        next_pair.as_rule()
                    ))?;
                }
            }
            Ok(AstNode::ConstInitVal(ast::ConstInitValType::InitList(
                init_vals,
            )))
        } else {
            Ok(AstNode::ConstInitVal(ast::ConstInitValType::ConstExp(
                AstBuilder::build_ast(inner.next().unwrap())?,
            )))
        }
    }

    fn build_var_decl(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let btype = AstBuilder::build_ast(inner.next().unwrap())?;
        let var_defs = inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(AstBuilder::build_ast)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::VarDecl { btype, var_defs })
    }

    // WIP
    fn build_var_def(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_str().to_string();
        let mut dimensions = Vec::new();
        let mut init_val = None;

        while let Some(next_pair) = inner.peek() {
            match next_pair.as_rule() {
                Rule::LBracket => {
                    inner.next(); // consume '['
                    dimensions.push(AstBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                    inner.next(); // consume ']'
                }
                Rule::Assign => {
                    inner.next(); // consume '='
                    init_val = Some(AstBuilder::build_ast(inner.next().unwrap())?);
                    break;
                }
                _ => break,
            }
        }

        Ok(AstNode::VarDef {
            ident,
            dimensions,
            init_val,
        })
    }

    fn build_init_val(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
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
                    init_vals.push(AstBuilder::build_ast(inner.next().unwrap())?);
                } else if next_pair.as_rule() == Rule::Comma {
                    inner.next(); // consume ';'
                } else {
                    unreachable!()
                }
            }
            Ok(AstNode::InitVal(ast::InitValType::List(init_vals)))
        } else {
            let exp = AstBuilder::build_ast(inner.next().unwrap())?;
            Ok(AstNode::InitVal(ast::InitValType::Exp(exp)))
        }
    }

    fn build_func_def(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let func_type = AstBuilder::build_ast(inner.next().unwrap())?;
        let ident = inner.next().unwrap().as_str().to_string();
        inner.next(); // consume '('
        let mut params = None;
        if let Some(next_pair) = inner.peek() {
            if next_pair.as_rule() == Rule::FuncFParams {
                params = Some(AstBuilder::build_ast(inner.next().unwrap())?);
            }
        }
        inner.next(); // consume ')'
        let body = AstBuilder::build_ast(inner.next().unwrap())?;
        Ok(AstNode::FuncDef {
            func_type,
            ident,
            params,
            body,
        })
    }

    fn build_func_type(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let func_type = pair.as_str().to_string();
        Ok(AstNode::FuncType(func_type))
    }

    fn build_func_fparams(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let params = pair
            .into_inner()
            .filter(|p| p.as_rule() != Rule::Comma)
            .map(AstBuilder::build_ast)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::FuncFParams(params))
    }

    fn build_func_fparam(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let btype = AstBuilder::build_ast(inner.next().unwrap())?;
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
                        dimensions.push(AstBuilder::build_ast(inner.next().unwrap())?); // IntegerConst
                        inner.next(); // consume ']'
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(AstNode::FuncFParam {
            btype,
            ident,
            is_array,
            dimensions,
        })
    }

    fn build_block(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let items = pair
            .into_inner()
            .filter(|p| p.as_rule() != Rule::LBrace && p.as_rule() != Rule::RBrace)
            .map(AstBuilder::build_ast)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::Block(items))
    }

    fn build_block_item(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::BlockItem(AstBuilder::build_ast(
            pair.into_inner().next().unwrap(),
        )?))
    }

    fn build_stmt(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_cond(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_lval(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_primary_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_number(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_unary_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_unary_op(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_func_rparams(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_mul_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_add_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_rel_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_eq_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_and_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_or_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
    }

    fn build_const_exp(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        unimplemented!()
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
            AstBuilder::display_ast_inner(p, new_prefix, is_last_child);
        }
    }

    fn display_ast(pair: pest::iterators::Pair<Rule>) {
        AstBuilder::display_ast_inner(pair, "".to_string(), true);
    }
}

pub fn display_ast(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::prog, input).map_err(|e| format!("{}", e))?;
    AstBuilder::display_ast(parse_result.into_iter().next().unwrap());
    Ok(())
}

pub fn parse(input: &str) -> Result<(), String> {
    let parse_result = SysYParser::parse(Rule::prog, input).map_err(|e| format!("{}", e))?;
    let _ = AstBuilder::build_ast(parse_result.into_iter().next().unwrap());
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
