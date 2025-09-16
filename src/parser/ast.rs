#![allow(dead_code)]

use crate::parser::Parser;
use crate::parser::Rule;

#[derive(Debug)]
pub struct AstNode {
    inner: AstNodeInner,
    line_col: (usize, usize),
}

impl AstNode {
    pub fn new(inner: AstNodeInner, line_col: (usize, usize)) -> Self {
        Self { inner, line_col }
    }

    pub fn as_inner(&self) -> &AstNodeInner {
        &self.inner
    }

    pub fn into_inner(self) -> AstNodeInner {
        self.inner
    }

    pub fn line_col(&self) -> (usize, usize) {
        self.line_col
    }
}

#[derive(Debug)]
pub enum AstNodeInner {
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
    ParseError,
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
