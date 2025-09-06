#![allow(dead_code)]

use crate::parser::AstBuilder;
use crate::parser::Rule;

#[derive(Debug)]
pub enum AstNode {
    CompUnit(Vec<Box<AstNode>>),

    Decl(Box<AstNode>),

    BType(String),

    ConstDecl {
        btype: Box<AstNode>,
        const_defs: Vec<Box<AstNode>>,
    },
    ConstDef {
        ident: String,
        dimensions: Vec<Box<AstNode>>,
        init_val: Box<AstNode>,
    },
    ConstInitVal(ConstInitValType),

    VarDecl {
        btype: Box<AstNode>,
        var_defs: Vec<Box<AstNode>>,
    },
    VarDef {
        ident: String,
        dimensions: Vec<Box<AstNode>>,
        init_val: Option<Box<AstNode>>,
    },
    InitVal(InitValType),

    FuncDef {
        func_type: Box<AstNode>,
        ident: String,
        params: Option<Box<AstNode>>,
        body: Box<AstNode>,
    },
    FuncType(String),
    FuncFParams(Vec<Box<AstNode>>),
    FuncFParam {
        btype: Box<AstNode>,
        ident: String,
        is_array: bool,
        dimensions: Vec<Box<AstNode>>,
    },

    Block(Vec<Box<AstNode>>),
    BlockItem(Box<AstNode>),

    Stmt(Box<StmtType>),

    Exp(Box<AstNode>),
    Cond(Box<AstNode>),
    LVal {
        ident: String,
        dimensions: Vec<Box<AstNode>>,
    },
    PrimaryExp(PrimaryExpType),
    Number(NumberType),
    UnaryExp(UnaryExpType),
    UnaryOp(UnaryOpType),
    FuncRParams(Vec<Box<AstNode>>),
    MulExp {
        lhs: Box<AstNode>,
        ops: Vec<(MulOpType, Box<AstNode>)>,
    },
    AddExp {
        lhs: Box<AstNode>,
        ops: Vec<(AddOpType, Box<AstNode>)>,
    },
    RelExp {
        lhs: Box<AstNode>,
        ops: Vec<(RelOpType, Box<AstNode>)>,
    },
    EqExp {
        lhs: Box<AstNode>,
        ops: Vec<(EqOpType, Box<AstNode>)>,
    },
    AndExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpType, Box<AstNode>)>,
    },
    OrExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpType, Box<AstNode>)>,
    },
    ConstExp(Box<AstNode>),
}

impl<'a> TryFrom<pest::iterators::Pair<'a, Rule>> for AstNode {
    type Error = String;

    fn try_from(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::CompUnit => AstBuilder::build_comp_unit(pair),
            Rule::Decl => AstBuilder::build_decl(pair),
            Rule::BType => AstBuilder::build_btype(pair),
            Rule::ConstDecl => AstBuilder::build_const_decl(pair),
            Rule::ConstDef => AstBuilder::build_const_def(pair),
            Rule::ConstInitVal => AstBuilder::build_const_init_val(pair),
            Rule::VarDecl => AstBuilder::build_var_decl(pair),
            Rule::VarDef => AstBuilder::build_var_def(pair),
            Rule::InitVal => AstBuilder::build_init_val(pair),
            Rule::FuncDef => AstBuilder::build_func_def(pair),
            Rule::FuncType => AstBuilder::build_func_type(pair),
            Rule::FuncFParams => AstBuilder::build_func_fparams(pair),
            Rule::FuncFParam => AstBuilder::build_func_fparam(pair),
            Rule::Block => AstBuilder::build_block(pair),
            Rule::BlockItem => AstBuilder::build_block_item(pair),
            Rule::Stmt => AstBuilder::build_stmt(pair),
            Rule::Exp => AstBuilder::build_exp(pair),
            Rule::Cond => AstBuilder::build_cond(pair),
            Rule::LVal => AstBuilder::build_lval(pair),
            Rule::PrimaryExp => AstBuilder::build_primary_exp(pair),
            Rule::Number => AstBuilder::build_number(pair),
            Rule::UnaryExp => AstBuilder::build_unary_exp(pair),
            Rule::UnaryOp => AstBuilder::build_unary_op(pair),
            Rule::FuncRParams => AstBuilder::build_func_rparams(pair),
            Rule::MulExp => AstBuilder::build_mul_exp(pair),
            Rule::AddExp => AstBuilder::build_add_exp(pair),
            Rule::RelExp => AstBuilder::build_rel_exp(pair),
            Rule::EqExp => AstBuilder::build_eq_exp(pair),
            Rule::AndExp => AstBuilder::build_and_exp(pair),
            Rule::OrExp => AstBuilder::build_or_exp(pair),
            Rule::ConstExp => AstBuilder::build_const_exp(pair),
            _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
        }
    }
}

#[derive(Debug)]
pub enum UnaryExpType {
    FuncCall {
        ident: String,
        args: Option<Vec<Box<AstNode>>>,
    },
    PrimaryExp(Box<AstNode>),
    Unary {
        op: Box<AstNode>,
        exp: Box<AstNode>,
    },
}

#[derive(Debug)]
pub enum UnaryOpType {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum MulOpType {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum AddOpType {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum RelOpType {
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
pub enum EqOpType {
    Eq,
    Neq,
}

#[derive(Debug)]
pub enum LogicOpType {
    And,
    Or,
}

#[derive(Debug)]
pub enum PrimaryExpType {
    Exp(Box<AstNode>),
    LVal(Box<AstNode>),
    Number(Box<AstNode>),
}

#[derive(Debug)]
pub enum StmtType {
    Assign {
        lval: Box<AstNode>,
        exp: Box<AstNode>,
    },
    Exp(Option<Box<AstNode>>),
    Block(Box<AstNode>),
    If {
        cond: Box<AstNode>,
        then_stmt: Box<AstNode>,
        else_stmt: Option<Box<AstNode>>,
    },
    While {
        cond: Box<AstNode>,
        stmt: Box<AstNode>,
    },
    Break,
    Continue,
    Return(Option<Box<AstNode>>),
}

#[derive(Debug)]
pub enum ConstInitValType {
    ConstExp(Box<AstNode>),
    InitList(Vec<Box<AstNode>>),
}

#[derive(Debug)]
pub enum InitValType {
    ConstExp(Box<AstNode>),
    InitList(Vec<Box<AstNode>>),
}

#[derive(Debug)]
pub enum NumberType {
    IntegerConst(String),
}
