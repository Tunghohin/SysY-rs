#[derive(Debug)]
pub enum AstNode {
    CompUnit(Vec<Box<AstNode>>),
    ConstDecl {
        btype: Box<AstNode>,
        const_defs: Vec<Box<AstNode>>,
    },
}