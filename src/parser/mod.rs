#![allow(dead_code)]

pub mod ast;

use crate::semantic::{
    self, SemanticChecker,
    symbol_table::{self, FunctionType, Type},
};
use ast::AstNodeInner;
use pest::{Parser, error::ErrorVariant};
use pest_derive::Parser;

pub use ast::*;

#[derive(Parser)]
#[grammar = "./rules.pest"]
pub(crate) struct SysYParser;

#[derive(Debug, Default)]
pub struct BuildConfig {}

#[derive(Debug)]
pub struct ErrorCollector<F, P>
where
    F: Fn(usize, &P) -> String,
{
    errors: Vec<String>,
    formatter: F,
    _phantom: std::marker::PhantomData<P>,
}

impl<F, P> ErrorCollector<F, P>
where
    F: Fn(usize, &P) -> String,
{
    pub fn new(formatter: F) -> Self {
        Self {
            errors: Vec::new(),
            formatter,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn push_error(&mut self, line: usize, error: P) {
        let msg = (self.formatter)(line, &error);
        self.errors.push(msg);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::replace(&mut self.errors, Vec::new())
    }
}

pub struct AstBuilder {
    config: BuildConfig,
    parse_errors: ErrorCollector<fn(usize, &String) -> String, String>,
    semantic_errors:
        ErrorCollector<fn(usize, &semantic::SemanticError) -> String, semantic::SemanticError>,
    semantic_checker: SemanticChecker,
}

impl AstBuilder {
    pub fn new(config: BuildConfig) -> Self {
        AstBuilder {
            config,
            parse_errors: ErrorCollector::new(|line, e| {
                format!("Error type {} at Line {}: {}", "B", line, e.trim_end())
            }),
            semantic_errors: ErrorCollector::new(|line, e| {
                let typeid: i32 = e.clone().into();
                let msg: String = e.clone().into();
                format!("Error type {} at Line {}: {}", typeid, line, msg)
            }),
            semantic_checker: SemanticChecker::default(),
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
        if self.parse_errors.has_error() {
            return Err(self.parse_errors.take_errors().join("\n"));
        }
        if self.semantic_errors.has_error() {
            return Err(self.semantic_errors.take_errors().join("\n"));
        }
        root
    }

    fn build_ast_node(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<Box<AstNode>, String> {
        let node = match pair.as_rule() {
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
            Rule::TopError | Rule::DeclError | Rule::StmtError => {
                let rule = match pair.as_rule() {
                    Rule::TopError => "TopError".to_string(),
                    Rule::DeclError => "DeclError".to_string(),
                    Rule::StmtError => "StmtError".to_string(),
                    _ => "Unknown".to_string(),
                };
                let line_col = pair.line_col();
                let inner = AstNodeInner::ParseError;
                self.parse_errors.push_error(line_col.0, rule);
                Ok(Box::new(AstNode::new(inner, line_col)))
            }
            _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
        };

        node.and_then(|node| {
            match node.as_inner() {
                AstNodeInner::Stmt(_stmt_inner) => {
                    if let Err(e) = self.semantic_checker.check(&node) {
                        self.semantic_errors.push_error(node.line_col().0, e);
                    }
                }
                AstNodeInner::ConstDecl {
                    btype: _,
                    const_defs: _,
                } => {
                    if let Err(e) = self.semantic_checker.check(&node) {
                        self.semantic_errors.push_error(node.line_col().0, e);
                    }
                }
                AstNodeInner::VarDecl {
                    btype: _,
                    var_defs: _,
                } => {
                    if let Err(e) = self.semantic_checker.check(&node) {
                        self.semantic_errors.push_error(node.line_col().0, e);
                    }
                }
                AstNodeInner::FuncDef {
                    func_type: _,
                    ident: _,
                    params: _,
                    body: _,
                } => {
                    // skip semantic check for function definition here, as it has been done before building body
                }
                _ => {}
            };
            Ok(node)
        })
        .or_else(|e| Err(e))
    }

    fn build_comp_unit(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::CompUnit(
            pair.into_inner()
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(AstNode::new(inner, line_col))
    }

    fn build_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::Decl(self.build_ast_node(pair.into_inner().next().unwrap())?);
        Ok(AstNode::new(inner, line_col))
    }

    fn build_btype(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::BType(pair.as_str().to_string());
        Ok(AstNode::new(inner, line_col))
    }

    fn build_const_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        pair_inner.next(); // consume 'Const'
        let btype = self.build_ast_node(pair_inner.next().unwrap())?;
        let const_defs = pair_inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        let inner = AstNodeInner::ConstDecl { btype, const_defs };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_const_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let ident = pair_inner.next().unwrap().as_str().to_string();
        let mut dimensions = Vec::new();
        while let Some(next_pair) = pair_inner.peek() {
            if next_pair.as_rule() == Rule::LBracket {
                pair_inner.next(); // consume '['
                dimensions.push(self.build_ast_node(pair_inner.next().unwrap())?); // ConstExp
                pair_inner.next(); // consume ']'
            } else {
                break;
            }
        }
        pair_inner.next(); // consume '='
        let init_val = self.build_ast_node(pair_inner.next().unwrap())?;

        let inner = AstNodeInner::ConstDef {
            ident,
            dimensions,
            init_val,
        };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_const_init_val(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let mut pair_inner = pair.into_inner();
        match pair_inner.peek().unwrap().as_rule() {
            Rule::ConstExp => {
                let inner = AstNodeInner::ConstInitVal(ast::ConstInitValInner::ConstExp(
                    self.build_ast_node(pair_inner.next().unwrap())?,
                ));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::LBrace => {
                pair_inner.next(); // consume '{'
                let mut init_vals = Vec::new();
                while let Some(next_pair) = pair_inner.peek() {
                    if next_pair.as_rule() == Rule::RBrace {
                        break;
                    }
                    if next_pair.as_rule() != Rule::Comma {
                        init_vals.push(self.build_ast_node(pair_inner.next().unwrap())?);
                    } else if next_pair.as_rule() == Rule::Comma {
                        pair_inner.next(); // consume ','
                    } else {
                        return Err(format!(
                            "Unexpected rule in ConstInitVal: {:?}",
                            next_pair.as_rule()
                        ))?;
                    }
                }
                let inner = AstNodeInner::ConstInitVal(ast::ConstInitValInner::InitList(init_vals));
                Ok(AstNode::new(inner, line_col))
            }
            _ => Err(format!(
                "Unexpected rule in ConstInitVal: {:?}",
                pair_inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_var_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut inner = pair.into_inner();
        let btype = self.build_ast_node(inner.next().unwrap())?;
        let var_defs = inner
            .filter(|p| p.as_rule() != Rule::Comma && p.as_rule() != Rule::Semicolon)
            .map(|p| self.build_ast_node(p))
            .collect::<Result<Vec<_>, _>>()?;
        let inner = AstNodeInner::VarDecl { btype, var_defs };
        Ok(AstNode::new(inner, line_col))
    }

    fn build_var_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let ident = pair_inner.next().unwrap().as_str().to_string();
        let mut dimensions = Vec::new();
        let mut init_val = None;
        while let Some(next_pair) = pair_inner.peek() {
            match next_pair.as_rule() {
                Rule::LBracket => {
                    pair_inner.next(); // consume '['
                    dimensions.push(self.build_ast_node(pair_inner.next().unwrap())?); // ConstExp
                    pair_inner.next(); // consume ']'
                }
                Rule::Assign => {
                    pair_inner.next(); // consume '='
                    init_val = Some(self.build_ast_node(pair_inner.next().unwrap())?);
                    break;
                }
                _ => break,
            }
        }
        let inner = AstNodeInner::VarDef {
            ident,
            dimensions,
            init_val,
        };
        Ok(AstNode::new(inner, line_col))
    }

    fn build_init_val(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        match pair_inner.peek().unwrap().as_rule() {
            Rule::LBrace => {
                pair_inner.next(); // consume '{'
                let mut init_vals = Vec::new();
                while let Some(next_pair) = pair_inner.peek() {
                    if next_pair.as_rule() == Rule::RBrace {
                        break;
                    }
                    if next_pair.as_rule() != Rule::Comma {
                        init_vals.push(self.build_ast_node(pair_inner.next().unwrap())?);
                    } else if next_pair.as_rule() == Rule::Comma {
                        pair_inner.next(); // consume ','
                    } else {
                        return Err(format!(
                            "Unexpected rule in init_val: {:?}",
                            next_pair.as_rule()
                        ))?;
                    }
                }
                let inner = AstNodeInner::InitVal(ast::InitValInner::InitList(init_vals));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::Exp => {
                let inner = AstNodeInner::InitVal(ast::InitValInner::ConstExp(
                    self.build_ast_node(pair_inner.next().unwrap())?,
                ));
                Ok(AstNode::new(inner, line_col))
            }
            _ => Err(format!(
                "Unexpected rule in init_val: {:?}",
                pair_inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_func_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let func_type = self.build_ast_node(pair_inner.next().unwrap())?;
        let ident = pair_inner.next().unwrap().as_str().to_string();

        pair_inner.next(); // consume '('
        let mut params = None;
        if let Some(next_pair) = pair_inner.peek() {
            if next_pair.as_rule() == Rule::FuncFParams {
                params = Some(self.build_ast_node(pair_inner.next().unwrap())?);
            }
        }
        pair_inner.next(); // consume ')'

        // define function in symbol table
        let ret_ty = match func_type.as_inner() {
            AstNodeInner::FuncType(s) if s == "int" => Type::Int,
            AstNodeInner::FuncType(s) if s == "void" => Type::Void,
            _ => {
                return Err(format!(
                    "Invalid function return type: {:?}",
                    func_type.as_inner()
                ));
            }
        };
        let mut params_ty = Vec::new();
        if let Some(params) = params.as_ref() {
            if let AstNodeInner::FuncFParams(param_list) = params.as_inner() {
                for param in param_list {
                    match param.as_inner() {
                        AstNodeInner::FuncFParam {
                            btype,
                            ident: _,
                            is_array,
                            dimensions: _,
                        } => {
                            let ty = match btype.as_inner() {
                                AstNodeInner::BType(s) if s == "int" => {
                                    if *is_array {
                                        Type::Array(symbol_table::ArrayType {
                                            ty: Box::new(Type::Int),
                                            num_elements: None,
                                        })
                                    } else {
                                        Type::Int
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            };
                            params_ty.push(Box::new(ty));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        let func_ty = FunctionType {
            ret_ty: Box::new(ret_ty),
            params_ty,
        };
        if self
            .semantic_checker
            .scope_stk
            .peek_mut()
            .unwrap_or_else(|| unreachable!())
            .define(
                &ident,
                symbol_table::VariableMetadata {
                    ty: Type::Function(func_ty),
                },
            )
            .is_err()
        {
            self.semantic_errors.push_error(
                line_col.0,
                semantic::SemanticError::RedefinedFunction(ident.clone()),
            );
        }

        let ret_ty = match func_type.as_inner() {
            AstNodeInner::FuncType(s) if s == "int" => Type::Int,
            AstNodeInner::FuncType(s) if s == "void" => Type::Void,
            _ => {
                return Err(format!(
                    "Invalid function return type: {:?}",
                    func_type.as_inner()
                ));
            }
        };
        self.semantic_checker
            .entry_scope(Some(Box::new(ret_ty.clone())));
        // define parameters in symbol table
        if let Some(params) = params.as_ref() {
            if let AstNodeInner::FuncFParams(param_list) = params.as_inner() {
                for param in param_list {
                    if let AstNodeInner::FuncFParam {
                        btype,
                        ident,
                        is_array,
                        dimensions: _,
                    } = param.as_inner()
                    {
                        let ty = match btype.as_inner() {
                            AstNodeInner::BType(s) if s == "int" => {
                                if *is_array {
                                    Type::Array(symbol_table::ArrayType {
                                        ty: Box::new(Type::Int),
                                        num_elements: None,
                                    })
                                } else {
                                    Type::Int
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        };
                        if self
                            .semantic_checker
                            .scope_stk
                            .peek_mut()
                            .unwrap_or_else(|| unreachable!())
                            .define(ident, symbol_table::VariableMetadata { ty: ty.clone() })
                            .is_err()
                        {
                            self.semantic_errors.push_error(
                                line_col.0,
                                semantic::SemanticError::RedefinedVariable(ident.clone()),
                            );
                        }
                    }
                }
            }
        }

        let body = self.build_ast_node(pair_inner.next().unwrap())?;
        let inner = AstNodeInner::FuncDef {
            func_type,
            ident,
            params,
            body,
        };

        self.semantic_checker.exit_scope();
        Ok(AstNode::new(inner, line_col))
    }

    fn build_func_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::FuncType(pair.as_str().to_string());
        Ok(AstNode::new(inner, line_col))
    }

    fn build_func_fparams(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::FuncFParams(
            pair.into_inner()
                .filter(|p| p.as_rule() != Rule::Comma)
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(AstNode::new(inner, line_col))
    }

    fn build_func_fparam(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let btype = self.build_ast_node(pair_inner.next().unwrap())?;
        let ident = pair_inner.next().unwrap().as_str().to_string();
        let mut is_array = false;
        let mut dimensions = Vec::new();
        if let Some(next_pair) = pair_inner.peek() {
            if next_pair.as_rule() == Rule::LBracket {
                pair_inner.next(); // consume '['
                pair_inner.next(); // consume ']'
                is_array = true;
                while let Some(bracket) = pair_inner.peek() {
                    if bracket.as_rule() == Rule::LBracket {
                        pair_inner.next(); // consume '['
                        dimensions.push(self.build_ast_node(pair_inner.next().unwrap())?); // ConstExp
                        pair_inner.next(); // consume ']'
                    } else {
                        break;
                    }
                }
            }
        }
        let inner = AstNodeInner::FuncFParam {
            btype,
            ident,
            is_array,
            dimensions,
        };
        Ok(AstNode::new(inner, line_col))
    }

    fn build_block(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        self.semantic_checker
            .entry_scope(match self.semantic_checker.return_type() {
                Some(ty) => Some(Box::new(ty.clone())),
                None => None,
            });
        let line_col = pair.line_col();
        let inner = AstNodeInner::Block(
            pair.into_inner()
                .filter(|p| p.as_rule() != Rule::LBrace && p.as_rule() != Rule::RBrace)
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        );
        self.semantic_checker.exit_scope();
        Ok(AstNode::new(inner, line_col))
    }

    fn build_block_item(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner =
            AstNodeInner::BlockItem(self.build_ast_node(pair.into_inner().next().unwrap())?);
        Ok(AstNode::new(inner, line_col))
    }

    fn build_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        match pair_inner.peek().unwrap().as_rule() {
            Rule::LVal => {
                let inner = AstNodeInner::Stmt({
                    let lval = self.build_ast_node(pair_inner.next().unwrap())?;
                    pair_inner.next(); // consume '='
                    let exp = self.build_ast_node(pair_inner.next().unwrap())?;
                    Box::new(ast::StmtInner::Assign { lval, exp })
                });
                Ok(AstNode::new(inner, line_col))
            }
            Rule::Block => {
                let inner = AstNodeInner::Stmt(Box::new(ast::StmtInner::Block(
                    self.build_ast_node(pair_inner.next().unwrap())?,
                )));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::If => {
                let inner = AstNodeInner::Stmt(Box::new({
                    pair_inner.next(); // consume 'if'
                    pair_inner.next(); // consume '('
                    let cond = self.build_ast_node(pair_inner.next().unwrap())?;
                    pair_inner.next(); // consume ')'
                    let then_stmt = self.build_ast_node(pair_inner.next().unwrap())?;
                    let else_stmt = if let Some(else_token) = pair_inner.peek() {
                        if else_token.as_rule() == Rule::Else {
                            pair_inner.next(); // consume 'else'
                            Some(self.build_ast_node(pair_inner.next().unwrap())?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    ast::StmtInner::If {
                        cond,
                        then_stmt,
                        else_stmt,
                    }
                }));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::While => {
                let inner = AstNodeInner::Stmt(Box::new({
                    pair_inner.next(); // consume 'while'
                    pair_inner.next(); // consume '('
                    let cond = self.build_ast_node(pair_inner.next().unwrap())?;
                    pair_inner.next(); // consume ')'
                    let stmt = self.build_ast_node(pair_inner.next().unwrap())?;
                    ast::StmtInner::While { cond, stmt }
                }));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::Break => {
                let inner = AstNodeInner::Stmt(Box::new(ast::StmtInner::Break));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::Continue => {
                let inner = AstNodeInner::Stmt(Box::new(ast::StmtInner::Continue));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::Return => {
                let inner = AstNodeInner::Stmt(Box::new({
                    pair_inner.next(); // consume 'return'
                    let exp = if let Some(next_pair) = pair_inner.peek() {
                        if next_pair.as_rule() != Rule::Semicolon {
                            Some(self.build_ast_node(pair_inner.next().unwrap())?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    ast::StmtInner::Return(exp)
                }));
                Ok(AstNode::new(inner, line_col))
            }
            _ => {
                let exp = if let Some(next_pair) = pair_inner.peek() {
                    if next_pair.as_rule() != Rule::Semicolon {
                        Some(self.build_ast_node(pair_inner.next().unwrap())?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let inner = AstNodeInner::Stmt(Box::new(ast::StmtInner::Exp(exp)));
                Ok(AstNode::new(inner, line_col))
            }
        }
    }

    fn build_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::Exp(self.build_ast_node(pair.into_inner().next().unwrap())?);
        Ok(AstNode::new(inner, line_col))
    }

    fn build_cond(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::Cond(self.build_ast_node(pair.into_inner().next().unwrap())?);
        Ok(AstNode::new(inner, line_col))
    }

    fn build_lval(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let inner = AstNodeInner::LVal {
            ident: pair_inner.next().unwrap().as_str().to_string(),
            dimensions: pair_inner
                .filter(|p| p.as_rule() != Rule::LBracket && p.as_rule() != Rule::RBracket)
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_primary_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let mut pair_inner = pair.into_inner();
        if let Some(next_pair) = pair_inner.peek() {
            match next_pair.as_rule() {
                Rule::LVal => {
                    let inner = AstNodeInner::PrimaryExp(ast::PrimaryExpInner::LVal(
                        self.build_ast_node(pair_inner.next().unwrap())?,
                    ));
                    Ok(AstNode::new(inner, line_col))
                }
                Rule::Number => {
                    let inner = AstNodeInner::PrimaryExp(ast::PrimaryExpInner::Number(
                        self.build_ast_node(pair_inner.next().unwrap())?,
                    ));
                    Ok(AstNode::new(inner, line_col))
                }
                Rule::LParen => {
                    pair_inner.next(); // consume '('
                    let exp = self.build_ast_node(pair_inner.next().unwrap())?;
                    pair_inner.next(); // consume ')'
                    let inner = AstNodeInner::PrimaryExp(ast::PrimaryExpInner::Exp(exp));
                    Ok(AstNode::new(inner, line_col))
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
        let line_col = pair.line_col();
        let inner = AstNodeInner::Number(ast::NumberInner::IntegerConst(pair.as_str().to_string()));
        Ok(AstNode::new(inner, line_col))
    }

    fn build_unary_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let mut pair_inner = pair.into_inner();
        match pair_inner.peek().unwrap().as_rule() {
            Rule::Ident => {
                let ident = pair_inner.next().unwrap().as_str().to_string();
                pair_inner.next(); // consume '('
                let args = if let Some(next_pair) = pair_inner.peek() {
                    match next_pair.as_rule() {
                        Rule::FuncRParams => Some(
                            self.build_ast_node(pair_inner.next().unwrap())
                                .map(|node| match node.into_inner() {
                                    AstNodeInner::FuncRParams(params) => params,
                                    _ => vec![],
                                })
                                .map_err(|e| e)?,
                        ),
                        _ => None,
                    }
                } else {
                    None
                };
                pair_inner.next(); // consume ')'

                let inner = AstNodeInner::UnaryExp(ast::UnaryExpInner::FuncCall { ident, args });
                Ok(AstNode::new(inner, line_col))
            }
            Rule::PrimaryExp => {
                let inner = AstNodeInner::UnaryExp(ast::UnaryExpInner::PrimaryExp(
                    self.build_ast_node(pair_inner.next().unwrap())?,
                ));
                Ok(AstNode::new(inner, line_col))
            }
            Rule::UnaryOp => {
                let op = self.build_ast_node(pair_inner.next().unwrap())?;
                let exp = self.build_ast_node(pair_inner.next().unwrap())?;

                let inner = AstNodeInner::UnaryExp(ast::UnaryExpInner::Unary { op, exp });
                Ok(AstNode::new(inner, line_col))
            }
            _ => Err(format!(
                "Unexpected rule in UnaryExp: {:?}",
                pair_inner.peek().unwrap().as_rule()
            )),
        }
    }

    fn build_unary_op(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::UnaryOp({
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::Plus => ast::UnaryOpInner::Plus,
                Rule::Minus => ast::UnaryOpInner::Minus,
                Rule::Not => ast::UnaryOpInner::Not,
                _ => return Err(format!("Unexpected UnaryOp: {:?}", inner.as_rule())),
            }
        });
        Ok(AstNode::new(inner, line_col))
    }

    fn build_func_rparams(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::FuncRParams(
            pair.into_inner()
                .filter(|p| p.as_rule() != Rule::Comma)
                .map(|p| self.build_ast_node(p))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(AstNode::new(inner, line_col))
    }

    fn build_mul_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::Mul => ast::MulOpInner::Mul,
                Rule::Div => ast::MulOpInner::Div,
                Rule::Mod => ast::MulOpInner::Mod,
                _ => return Err(format!("Unexpected operator in MulExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::MulExp { lhs, ops };
        Ok(AstNode::new(inner, line_col))
    }

    fn build_add_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::Plus => ast::AddOpInner::Plus,
                Rule::Minus => ast::AddOpInner::Minus,
                _ => return Err(format!("Unexpected operator in AddExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::AddExp { lhs, ops };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_rel_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::Lt => ast::RelOpInner::Lt,
                Rule::Gt => ast::RelOpInner::Gt,
                Rule::Le => ast::RelOpInner::Le,
                Rule::Ge => ast::RelOpInner::Ge,
                _ => return Err(format!("Unexpected operator in RelExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::RelExp { lhs, ops };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_eq_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::Eq => ast::EqOpInner::Eq,
                Rule::Neq => ast::EqOpInner::Neq,
                _ => return Err(format!("Unexpected operator in EqExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::EqExp { lhs, ops };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_and_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::And => ast::LogicOpInner::And,
                _ => return Err(format!("Unexpected operator in AndExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::AndExp { lhs, ops };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_or_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();

        let mut pair_inner = pair.into_inner();
        let lhs = self.build_ast_node(pair_inner.next().unwrap())?;
        let mut ops = Vec::new();
        while let Some(op) = pair_inner.next() {
            let op_type = match op.as_rule() {
                Rule::Or => ast::LogicOpInner::Or,
                _ => return Err(format!("Unexpected operator in OrExp: {:?}", op.as_rule())),
            };
            let rhs = self.build_ast_node(pair_inner.next().unwrap())?;
            ops.push((op_type, rhs));
        }
        let inner = AstNodeInner::OrExp { lhs, ops };

        Ok(AstNode::new(inner, line_col))
    }

    fn build_const_exp(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<AstNode, String> {
        let line_col = pair.line_col();
        let inner = AstNodeInner::ConstExp(self.build_ast_node(pair.into_inner().next().unwrap())?);
        Ok(AstNode::new(inner, line_col))
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
fn test_display_ast() {
    let src = std::fs::read_to_string("./tests/codegen/test1.in").unwrap_or_default();
    let _ = display_ast(&src);
}

#[test]
fn test_semantic_single() {
    let src = std::fs::read_to_string("./tests/semantic/normal6.in").unwrap_or_default();
    let _ = parse(&src, BuildConfig::default()).map_err(|e| println!("{}", e));
}
