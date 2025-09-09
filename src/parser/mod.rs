#![allow(dead_code)]

pub mod ast;

use ast::AstNode;
use pest::{Parser, error::ErrorVariant, error::InputLocation, error::LineColLocation};
use pest_derive::Parser;

pub use ast::*;

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

#[derive(Debug, Default)]
pub struct BuildConfig {}

#[derive(Debug, Default)]
pub struct ParseErrorCollector {
    pub errors: Vec<String>,
}

impl ParseErrorCollector {
    pub fn push_error(&mut self, line: usize, _col: usize, error: String) {
        self.errors.push(format!(
            "Error type B at Line {}: {}",
            line,
            error.trim_end()
        ));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub struct AstBuilder {
    config: BuildConfig,
    error_collector: ParseErrorCollector,
}

impl AstBuilder {
    pub fn new(config: BuildConfig) -> Self {
        AstBuilder {
            config,
            error_collector: ParseErrorCollector::default(),
        }
    }

    pub fn build(&mut self, src: &str) -> Result<Box<AstNode>, String> {
        let parse_result = SysYParser::parse(Rule::prog, src).map_err(|e| format!("{}", e))?;
        let root = self.build_ast_node(
            parse_result
                .into_iter()
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap(),
        );
        if self.error_collector.is_empty() {
            root
        } else {
            Err(self.error_collector.errors.join("\n"))
        }
    }

    fn build_ast_node(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<Box<AstNode>, String> {
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

            Rule::CompUnit => self.build_comp_unit(pair).map(Box::new),
            Rule::Decl => self.build_decl(pair).map(Box::new),
            Rule::BType => self.build_btype(pair).map(Box::new),
            Rule::ConstDecl => self.build_const_decl(pair).map(Box::new),
            Rule::ConstDef => self.build_const_def(pair).map(Box::new),
            Rule::ConstInitVal => self.build_const_init_val(pair).map(Box::new),
            Rule::VarDecl => self.build_var_decl(pair).map(Box::new),
            Rule::VarDef => self.build_var_def(pair).map(Box::new),
            Rule::InitVal => self.build_init_val(pair).map(Box::new),
            Rule::FuncDef => self.build_func_def(pair).map(Box::new),
            Rule::FuncType => self.build_func_type(pair).map(Box::new),
            Rule::FuncFParams => self.build_func_fparams(pair).map(Box::new),
            Rule::FuncFParam => self.build_func_fparam(pair).map(Box::new),
            Rule::Block => self.build_block(pair).map(Box::new),
            Rule::BlockItem => self.build_block_item(pair).map(Box::new),
            Rule::Stmt => self.build_stmt(pair).map(Box::new),
            Rule::Exp => self.build_exp(pair).map(Box::new),
            Rule::Cond => self.build_cond(pair).map(Box::new),
            Rule::LVal => self.build_lval(pair).map(Box::new),
            Rule::PrimaryExp => self.build_primary_exp(pair).map(Box::new),
            Rule::Number => self.build_number(pair).map(Box::new),
            Rule::UnaryExp => self.build_unary_exp(pair).map(Box::new),
            Rule::UnaryOp => self.build_unary_op(pair).map(Box::new),
            Rule::FuncRParams => self.build_func_rparams(pair).map(Box::new),
            Rule::MulExp => self.build_mul_exp(pair).map(Box::new),
            Rule::AddExp => self.build_add_exp(pair).map(Box::new),
            Rule::RelExp => self.build_rel_exp(pair).map(Box::new),
            Rule::EqExp => self.build_eq_exp(pair).map(Box::new),
            Rule::AndExp => self.build_and_exp(pair).map(Box::new),
            Rule::OrExp => self.build_or_exp(pair).map(Box::new),
            Rule::ConstExp => self.build_const_exp(pair).map(Box::new),
            Rule::ParseError => {
                self.error_collector.push_error(
                    pair.line_col().0,
                    pair.line_col().1,
                    pair.as_str().to_string(),
                );
                Ok(Box::new(AstNode::ParseError))
            }
            _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
        }
    }

    fn build_comp_unit(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::CompUnit(
            pair.into_inner()
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn build_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::Decl(
            self.build_ast_node(pair.into_inner().next().unwrap())?,
        ))
    }

    fn build_btype(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::BType(pair.as_str().to_string()))
    }

    fn build_const_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();

        inner.next(); // consume 'Const'

        let btype = self.build_ast_node(inner.next().unwrap())?;

        let const_defs = inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AstNode::ConstDecl { btype, const_defs })
    }

    fn build_const_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap().as_str().to_string();

        let mut dimensions = Vec::new();
        while let Some(next_pair) = inner.peek() {
            if next_pair.as_rule() == Rule::LBracket {
                inner.next(); // consume '['
                dimensions.push(self.build_ast_node(inner.next().unwrap())?); // ConstExp
                inner.next(); // consume ']'
            } else {
                break;
            }
        }

        inner.next(); // consume '='
        let init_val = self.build_ast_node(inner.next().unwrap())?;

        Ok(AstNode::ConstDef {
            ident,
            dimensions,
            init_val,
        })
    }

    fn build_const_init_val(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        match inner.peek().unwrap().as_rule() {
            Rule::ConstExp => Ok(AstNode::ConstInitVal(ast::ConstInitValType::ConstExp(
                self.build_ast_node(inner.next().unwrap())?,
            ))),
            Rule::LBrace => {
                inner.next(); // consume '{'
                let mut init_vals = Vec::new();
                while let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::RBrace {
                        break;
                    }
                    if next_pair.as_rule() != Rule::Comma {
                        init_vals.push(self.build_ast_node(inner.next().unwrap())?);
                    } else if next_pair.as_rule() == Rule::Comma {
                        inner.next(); // consume ','
                    } else {
                        return Err(format!(
                            "Unexpected rule in ConstInitVal: {:?}",
                            next_pair.as_rule()
                        ))?;
                    }
                }
                Ok(AstNode::ConstInitVal(ast::ConstInitValType::InitList(
                    init_vals,
                )))
            }
            _ => Err(format!(
                "Unexpected rule in ConstInitVal: {:?}",
                inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_var_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let btype = self.build_ast_node(inner.next().unwrap())?;
        let var_defs = inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::VarDecl { btype, var_defs })
    }

    fn build_var_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_str().to_string();
        let mut dimensions = Vec::new();
        let mut init_val = None;

        while let Some(next_pair) = inner.peek() {
            match next_pair.as_rule() {
                Rule::LBracket => {
                    inner.next(); // consume '['
                    dimensions.push(self.build_ast_node(inner.next().unwrap())?); // ConstExp
                    inner.next(); // consume ']'
                }
                Rule::Assign => {
                    inner.next(); // consume '='
                    init_val = Some(self.build_ast_node(inner.next().unwrap())?);
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

    fn build_init_val(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        match inner.peek().unwrap().as_rule() {
            Rule::LBrace => {
                inner.next(); // consume '{'
                let mut init_vals = Vec::new();
                while let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::RBrace {
                        break;
                    }
                    if next_pair.as_rule() != Rule::Comma {
                        init_vals.push(self.build_ast_node(inner.next().unwrap())?);
                    } else if next_pair.as_rule() == Rule::Comma {
                        inner.next(); // consume ','
                    } else {
                        return Err(format!(
                            "Unexpected rule in init_val: {:?}",
                            next_pair.as_rule()
                        ))?;
                    }
                }
                Ok(AstNode::InitVal(ast::InitValType::InitList(init_vals)))
            }
            Rule::Exp => Ok(AstNode::InitVal(ast::InitValType::ConstExp(
                self.build_ast_node(inner.next().unwrap())?,
            ))),
            _ => Err(format!(
                "Unexpected rule in init_val: {:?}",
                inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_func_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let func_type = self.build_ast_node(inner.next().unwrap())?;
        let ident = inner.next().unwrap().as_str().to_string();
        inner.next(); // consume '('
        let mut params = None;
        if let Some(next_pair) = inner.peek() {
            if next_pair.as_rule() == Rule::FuncFParams {
                params = Some(self.build_ast_node(inner.next().unwrap())?);
            }
        }
        inner.next(); // consume ')'
        let body = self.build_ast_node(inner.next().unwrap())?;
        Ok(AstNode::FuncDef {
            func_type,
            ident,
            params,
            body,
        })
    }

    fn build_func_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let func_type = pair.as_str().to_string();
        Ok(AstNode::FuncType(func_type))
    }

    fn build_func_fparams(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let params = pair
            .into_inner()
            .filter(|p| p.as_rule() != Rule::Comma)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::FuncFParams(params))
    }

    fn build_func_fparam(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let btype = self.build_ast_node(inner.next().unwrap())?;
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
                        dimensions.push(self.build_ast_node(inner.next().unwrap())?); // ConstExp
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

    fn build_block(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let items = pair
            .into_inner()
            .filter(|p| p.as_rule() != Rule::LBrace && p.as_rule() != Rule::RBrace)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::Block(items))
    }

    fn build_block_item(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::BlockItem(
            self.build_ast_node(pair.into_inner().next().unwrap())?,
        ))
    }

    fn build_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        match inner.peek().unwrap().as_rule() {
            Rule::LVal => {
                return Ok(AstNode::Stmt({
                    let lval = self.build_ast_node(inner.next().unwrap())?;
                    inner.next(); // consume '='
                    let exp = self.build_ast_node(inner.next().unwrap())?;
                    Box::new(ast::StmtType::Assign { lval, exp })
                }));
            }
            Rule::Block => {
                return Ok(AstNode::Stmt(Box::new(ast::StmtType::Block(
                    self.build_ast_node(inner.next().unwrap())?,
                ))));
            }
            Rule::If => {
                return Ok(AstNode::Stmt(Box::new({
                    inner.next(); // consume 'if'
                    inner.next(); // consume '('
                    let cond = self.build_ast_node(inner.next().unwrap())?;
                    inner.next(); // consume ')'
                    let then_stmt = self.build_ast_node(inner.next().unwrap())?;
                    let else_stmt = if let Some(else_token) = inner.peek() {
                        if else_token.as_rule() == Rule::Else {
                            inner.next(); // consume 'else'
                            Some(self.build_ast_node(inner.next().unwrap())?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    ast::StmtType::If {
                        cond,
                        then_stmt,
                        else_stmt,
                    }
                })));
            }
            Rule::While => {
                return Ok(AstNode::Stmt(Box::new({
                    inner.next(); // consume 'while'
                    inner.next(); // consume '('
                    let cond = self.build_ast_node(inner.next().unwrap())?;
                    inner.next(); // consume ')'
                    let stmt = self.build_ast_node(inner.next().unwrap())?;
                    ast::StmtType::While { cond, stmt }
                })));
            }
            Rule::Break => {
                return Ok(AstNode::Stmt(Box::new(ast::StmtType::Break)));
            }
            Rule::Continue => {
                return Ok(AstNode::Stmt(Box::new(ast::StmtType::Continue)));
            }
            Rule::Return => {
                return Ok(AstNode::Stmt(Box::new({
                    inner.next(); // consume 'return'
                    let exp = if let Some(next_pair) = inner.peek() {
                        if next_pair.as_rule() != Rule::Semicolon {
                            Some(self.build_ast_node(inner.next().unwrap())?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    ast::StmtType::Return(exp)
                })));
            }
            _ => {
                let exp = if let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() != Rule::Semicolon {
                        Some(self.build_ast_node(inner.next().unwrap())?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                return Ok(AstNode::Stmt(Box::new(ast::StmtType::Exp(exp))));
            }
        }
    }

    fn build_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::Exp(
            self.build_ast_node(pair.into_inner().next().unwrap())?,
        ))
    }

    fn build_cond(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::Cond(
            self.build_ast_node(pair.into_inner().next().unwrap())?,
        ))
    }

    fn build_lval(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        Ok(AstNode::LVal {
            ident: inner.next().unwrap().as_str().to_string(),
            dimensions: inner
                .filter(|p| p.as_rule() != Rule::LBracket && p.as_rule() != Rule::RBracket)
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn build_primary_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        if let Some(next_pair) = inner.peek() {
            match next_pair.as_rule() {
                Rule::LVal => Ok(AstNode::PrimaryExp(ast::PrimaryExpType::LVal(
                    self.build_ast_node(inner.next().unwrap())?,
                ))),
                Rule::Number => Ok(AstNode::PrimaryExp(ast::PrimaryExpType::Number(
                    self.build_ast_node(inner.next().unwrap())?,
                ))),
                Rule::LParen => {
                    inner.next(); // consume '('
                    let exp = self.build_ast_node(inner.next().unwrap())?;
                    inner.next(); // consume ')'
                    Ok(AstNode::PrimaryExp(ast::PrimaryExpType::Exp(exp)))
                }
                _ => Err(format!(
                    "Unexpected rule in PrimaryExp: {:?}",
                    next_pair.as_rule()
                )),
            }
        } else {
            Err("Empty PrimaryExp".to_string())
        }
    }

    fn build_number(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::Number(ast::NumberType::IntegerConst(
            pair.as_str().to_string(),
        )))
    }

    fn build_unary_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        match inner.peek().unwrap().as_rule() {
            Rule::Ident => {
                let ident = inner.next().unwrap().as_str().to_string();
                inner.next(); // consume '('
                let args = if let Some(next_pair) = inner.peek() {
                    match next_pair.as_rule() {
                        Rule::FuncRParams => Some(
                            self.build_ast_node(inner.next().unwrap())
                                .map(|node| match *node {
                                    AstNode::FuncRParams(params) => params,
                                    _ => vec![],
                                })
                                .map_err(|e| e)?,
                        ),
                        _ => None,
                    }
                } else {
                    None
                };
                inner.next(); // consume ')'
                Ok(AstNode::UnaryExp(ast::UnaryExpType::FuncCall {
                    ident,
                    args,
                }))
            }
            Rule::PrimaryExp => Ok(AstNode::UnaryExp(ast::UnaryExpType::PrimaryExp(
                self.build_ast_node(inner.next().unwrap())?,
            ))),
            Rule::UnaryOp => {
                let op = self.build_ast_node(inner.next().unwrap())?;
                let exp = self.build_ast_node(inner.next().unwrap())?;
                Ok(AstNode::UnaryExp(ast::UnaryExpType::Unary { op, exp }))
            }
            _ => Err(format!(
                "Unexpected rule in UnaryExp: {:?}",
                inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_unary_op(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::UnaryOp({
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::Plus => ast::UnaryOpType::Plus,
                Rule::Minus => ast::UnaryOpType::Minus,
                Rule::Not => ast::UnaryOpType::Not,
                _ => return Err(format!("Unexpected UnaryOp: {:?}", inner.as_rule())),
            }
        }))
    }

    fn build_func_rparams(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let params = pair
            .into_inner()
            .filter(|p| p.as_rule() != Rule::Comma)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AstNode::FuncRParams(params))
    }

    fn build_mul_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::Mul => ast::MulOpType::Mul,
                Rule::Div => ast::MulOpType::Div,
                Rule::Mod => ast::MulOpType::Mod,
                _ => return Err(format!("Unexpected operator in MulExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::MulExp { lhs, ops })
    }

    fn build_add_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::Plus => ast::AddOpType::Plus,
                Rule::Minus => ast::AddOpType::Minus,
                _ => return Err(format!("Unexpected operator in AddExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::AddExp { lhs, ops })
    }

    fn build_rel_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::Lt => ast::RelOpType::Lt,
                Rule::Gt => ast::RelOpType::Gt,
                Rule::Le => ast::RelOpType::Le,
                Rule::Ge => ast::RelOpType::Ge,
                _ => return Err(format!("Unexpected operator in RelExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::RelExp { lhs, ops })
    }

    fn build_eq_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::Eq => ast::EqOpType::Eq,
                Rule::Neq => ast::EqOpType::Neq,
                _ => return Err(format!("Unexpected operator in EqExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::EqExp { lhs, ops })
    }

    fn build_and_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::And => ast::LogicOpType::And,
                _ => return Err(format!("Unexpected operator in AndExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::AndExp { lhs, ops })
    }

    fn build_or_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let mut inner = pair.into_inner();
        let lhs = self.build_ast_node(inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = inner.next() {
            let op_type = match op.as_rule() {
                Rule::Or => ast::LogicOpType::Or,
                _ => return Err(format!("Unexpected operator in OrExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        Ok(AstNode::OrExp { lhs, ops })
    }

    fn build_const_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        Ok(AstNode::ConstExp(
            self.build_ast_node(pair.into_inner().next().unwrap())?,
        ))
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

fn pest_message_to_expecting(variant: &ErrorVariant<Rule>) -> String {
    match variant {
        ErrorVariant::ParsingError {
            positives,
            negatives: _,
        } => {
            let mut expected = Vec::new();
            for rule in positives {
                expected.push(format!("{:?}", rule));
            }
            expected.sort();
            expected.dedup();
            expected.join(", ")
        }
        _ => "Unknown parsing error".to_string(),
    }
}

pub fn parse(src: &str, config: BuildConfig) -> Result<Box<AstNode>, String> {
    let mut ast_builder = AstBuilder::new(config);
    ast_builder.build(src)
}

#[test]
fn test_parser() {
    let src = std::fs::read_to_string("./tests/parser/1.in").unwrap_or_default();
    match parse(&src, BuildConfig::default()) {
        Ok(root) => {
            println!("{:#?}", root);
        }
        Err(e) => {
            println!("{}", e);
        }
    };
}
