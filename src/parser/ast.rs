#![allow(dead_code)]

use crate::parser::Parser;
use crate::parser::Rule;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    ConstInitVal(ConstInitValInner),

    VarDecl {
        btype: Box<AstNode>,
        var_defs: Vec<Box<AstNode>>,
    },
    VarDef {
        ident: String,
        dimensions: Vec<Box<AstNode>>,
        init_val: Option<Box<AstNode>>,
    },
    InitVal(InitValInner),

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

    Stmt(Box<StmtInner>),

    Exp(Box<AstNode>),
    Cond(Box<AstNode>),
    LVal {
        ident: String,
        dimensions: Vec<Box<AstNode>>,
    },
    PrimaryExp(PrimaryExpInner),
    Number(NumberInner),
    UnaryExp(UnaryExpInner),
    UnaryOp(UnaryOpInner),
    FuncRParams(Vec<Box<AstNode>>),
    MulExp {
        lhs: Box<AstNode>,
        ops: Vec<(MulOpInner, Box<AstNode>)>,
    },
    AddExp {
        lhs: Box<AstNode>,
        ops: Vec<(AddOpInner, Box<AstNode>)>,
    },
    RelExp {
        lhs: Box<AstNode>,
        ops: Vec<(RelOpInner, Box<AstNode>)>,
    },
    EqExp {
        lhs: Box<AstNode>,
        ops: Vec<(EqOpInner, Box<AstNode>)>,
    },
    AndExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpInner, Box<AstNode>)>,
    },
    OrExp {
        lhs: Box<AstNode>,
        ops: Vec<(LogicOpInner, Box<AstNode>)>,
    },
    ConstExp(Box<AstNode>),
    ParseError,
}

#[derive(Debug, Clone)]
pub enum UnaryExpInner {
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

#[derive(Debug, Clone)]
pub enum UnaryOpInner {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone)]
pub enum MulOpInner {
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone)]
pub enum AddOpInner {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub enum RelOpInner {
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub enum EqOpInner {
    Eq,
    Neq,
}

#[derive(Debug, Clone)]
pub enum LogicOpInner {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum PrimaryExpInner {
    Exp(Box<AstNode>),
    LVal(Box<AstNode>),
    Number(Box<AstNode>),
}

#[derive(Debug, Clone)]
pub enum StmtInner {
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

#[derive(Debug, Clone)]
pub enum ConstInitValInner {
    ConstExp(Box<AstNode>),
    InitList(Vec<Box<AstNode>>),
}

#[derive(Debug, Clone)]
pub enum InitValInner {
    ConstExp(Box<AstNode>),
    InitList(Vec<Box<AstNode>>),
}

#[derive(Debug, Clone)]
pub enum NumberInner {
    IntegerConst(String),
}

impl TryInto<i32> for NumberInner {
    type Error = String;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            NumberInner::IntegerConst(s) => s
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse integer: {}", e)),
        }
    }
}
