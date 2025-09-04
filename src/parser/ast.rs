#![allow(dead_code)]

use std::ops::Add;

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
    LAndExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpType, Box<AstNode>)>,
    },
    LOrExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpType, Box<AstNode>)>,
    },
    ConstExp(Box<AstNode>),
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
    Exp(Box<AstNode>),
    List(Vec<Box<AstNode>>),
}

#[derive(Debug)]
pub enum InitValType {
    Exp(Box<AstNode>),
    List(Vec<Box<AstNode>>),
}

#[derive(Debug)]
pub enum NumberType {
    IntegerConst(i32),
}
