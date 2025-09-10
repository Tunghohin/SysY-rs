#![allow(dead_code)]

use crate::parser::{BuildConfig, ast::*};
use std::fmt::Write;
use std::fs;

pub struct Formatter {
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn format_ast(&mut self, ast: &AstNode) -> String {
        self.format_node(ast);
        self.output.trim_end().to_string()
    }

    fn write_indent(&mut self) {
        for _ in 0..(self.indent_level * 4) {
            self.output.push(' ');
        }
    }

    fn write_newline(&mut self) {
        self.output.push('\n');
    }

    fn write_str(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn format_node(&mut self, node: &AstNode) {
        match node {
            AstNode::CompUnit(children) => {
                let mut first_line = true;
                for (i, child) in children.iter().enumerate() {
                    if let AstNode::FuncDef { .. } = child.as_ref() {
                        if !first_line {
                            self.write_newline();
                        }
                    }

                    self.format_node(child);

                    if i < children.len() - 1 {
                        self.write_newline();
                    }
                    first_line = false;
                }
            }

            AstNode::Decl(child) => {
                self.format_node(child);
            }

            AstNode::ConstDecl { btype, const_defs } => {
                self.write_str("const ");
                self.format_node(btype);
                self.write_str(" ");
                for (i, def) in const_defs.iter().enumerate() {
                    if i > 0 {
                        self.write_str(", ");
                    }
                    self.format_node(def);
                }
                self.write_str(";");
            }

            AstNode::ConstDef {
                ident,
                dimensions,
                init_val,
            } => {
                self.write_str(ident);
                for dim in dimensions {
                    self.write_str("[");
                    self.format_node(dim);
                    self.write_str("]");
                }
                self.write_str(" = ");
                self.format_node(init_val);
            }

            AstNode::ConstInitVal(init_val_type) => match init_val_type {
                ConstInitValType::ConstExp(exp) => {
                    self.format_node(exp);
                }
                ConstInitValType::InitList(list) => {
                    self.write_str("{");
                    for (i, item) in list.iter().enumerate() {
                        if i > 0 {
                            self.write_str(", ");
                        }
                        self.format_node(item);
                    }
                    self.write_str("}");
                }
            },

            AstNode::VarDecl { btype, var_defs } => {
                self.format_node(btype);
                self.write_str(" ");
                for (i, def) in var_defs.iter().enumerate() {
                    if i > 0 {
                        self.write_str(", ");
                    }
                    self.format_node(def);
                }
                self.write_str(";");
            }

            AstNode::VarDef {
                ident,
                dimensions,
                init_val,
            } => {
                self.write_str(ident);
                for dim in dimensions {
                    self.write_str("[");
                    self.format_node(dim);
                    self.write_str("]");
                }
                if let Some(val) = init_val {
                    self.write_str(" = ");
                    self.format_node(val);
                }
            }

            AstNode::InitVal(init_val_type) => match init_val_type {
                InitValType::ConstExp(exp) => {
                    self.format_node(exp);
                }
                InitValType::InitList(list) => {
                    self.write_str("{");
                    for (i, item) in list.iter().enumerate() {
                        if i > 0 {
                            self.write_str(", ");
                        }
                        self.format_node(item);
                    }
                    self.write_str("}");
                }
            },

            AstNode::FuncDef {
                func_type,
                ident,
                params,
                body,
            } => {
                self.format_node(func_type);
                self.write_str(" ");
                self.write_str(ident);
                self.write_str("(");
                if let Some(p) = params {
                    self.format_node(p);
                }
                self.write_str(")");
                self.write_str(" ");
                self.format_node(body);
            }

            AstNode::FuncType(func_type) => {
                self.write_str(func_type);
            }

            AstNode::FuncFParams(params) => {
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write_str(", ");
                    }
                    self.format_node(param);
                }
            }

            AstNode::FuncFParam {
                btype,
                ident,
                is_array,
                dimensions,
            } => {
                self.format_node(btype);
                self.write_str(" ");
                self.write_str(ident);
                if *is_array {
                    self.write_str("[]");
                }
                for dim in dimensions {
                    self.write_str("[");
                    self.format_node(dim);
                    self.write_str("]");
                }
            }

            AstNode::Block(items) => {
                self.write_str("{");
                if !items.is_empty() {
                    self.write_newline();
                    self.indent_level += 1;

                    for item in items {
                        self.format_node(item);
                        self.write_newline();
                    }

                    self.indent_level -= 1;
                    self.write_indent();
                    self.write_str("}");
                } else {
                    self.write_newline();
                    self.write_indent();
                    self.write_str("}");
                }
            }

            AstNode::BlockItem(item) => {
                self.write_indent();
                self.format_node(item);
            }

            AstNode::Stmt(stmt) => {
                self.format_stmt_type(stmt);
            }

            AstNode::Exp(exp) => {
                self.format_node(exp);
            }

            AstNode::Cond(cond) => {
                self.format_node(cond);
            }

            AstNode::LVal { ident, dimensions } => {
                self.write_str(ident);
                for dim in dimensions {
                    self.write_str("[");
                    self.format_node(dim);
                    self.write_str("]");
                }
            }

            AstNode::PrimaryExp(primary_type) => match primary_type {
                PrimaryExpType::Exp(exp) => {
                    self.write_str("(");
                    self.format_node(exp);
                    self.write_str(")");
                }
                PrimaryExpType::LVal(lval) => {
                    self.format_node(lval);
                }
                PrimaryExpType::Number(number) => {
                    self.format_node(number);
                }
            },

            AstNode::Number(number_type) => match number_type {
                NumberType::IntegerConst(value) => {
                    write!(self.output, "{}", value).unwrap();
                }
            },

            AstNode::UnaryExp(unary_type) => match unary_type {
                UnaryExpType::FuncCall { ident, args } => {
                    self.write_str(ident);
                    self.write_str("(");
                    if let Some(arg_list) = args {
                        for (i, arg) in arg_list.iter().enumerate() {
                            if i > 0 {
                                self.write_str(", ");
                            }
                            self.format_node(arg);
                        }
                    }
                    self.write_str(")");
                }
                UnaryExpType::PrimaryExp(primary) => {
                    self.format_node(primary);
                }
                UnaryExpType::Unary { op, exp } => {
                    self.format_node(op);
                    self.format_node(exp);
                }
            },

            AstNode::UnaryOp(op_type) => match op_type {
                UnaryOpType::Plus => self.write_str("+"),
                UnaryOpType::Minus => self.write_str("-"),
                UnaryOpType::Not => self.write_str("!"),
            },

            AstNode::FuncRParams(args) => {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write_str(", ");
                    }
                    self.format_node(arg);
                }
            }

            AstNode::MulExp { lhs, ops } => {
                self.format_node(lhs);
                for (op, rhs) in ops {
                    self.write_str(" ");
                    match op {
                        MulOpType::Mul => self.write_str("*"),
                        MulOpType::Div => self.write_str("/"),
                        MulOpType::Mod => self.write_str("%"),
                    }
                    self.write_str(" ");
                    self.format_node(rhs);
                }
            }

            AstNode::AddExp { lhs, ops } => {
                self.format_node(lhs);
                for (op, rhs) in ops {
                    self.write_str(" ");
                    match op {
                        AddOpType::Plus => self.write_str("+"),
                        AddOpType::Minus => self.write_str("-"),
                    }
                    self.write_str(" ");
                    self.format_node(rhs);
                }
            }

            AstNode::RelExp { lhs, ops } => {
                self.format_node(lhs);
                for (op, rhs) in ops {
                    self.write_str(" ");
                    match op {
                        RelOpType::Lt => self.write_str("<"),
                        RelOpType::Gt => self.write_str(">"),
                        RelOpType::Le => self.write_str("<="),
                        RelOpType::Ge => self.write_str(">="),
                    }
                    self.write_str(" ");
                    self.format_node(rhs);
                }
            }

            AstNode::EqExp { lhs, ops } => {
                self.format_node(lhs);
                for (op, rhs) in ops {
                    self.write_str(" ");
                    match op {
                        EqOpType::Eq => self.write_str("=="),
                        EqOpType::Neq => self.write_str("!="),
                    }
                    self.write_str(" ");
                    self.format_node(rhs);
                }
            }

            AstNode::AndExp { lhs, ops } => {
                self.format_node(lhs);
                for (_, rhs) in ops {
                    self.write_str(" && ");
                    self.format_node(rhs);
                }
            }

            AstNode::OrExp { lhs, ops } => {
                self.format_node(lhs);
                for (_, rhs) in ops {
                    self.write_str(" || ");
                    self.format_node(rhs);
                }
            }

            AstNode::ConstExp(exp) => {
                self.format_node(exp);
            }

            AstNode::BType(btype) => {
                self.write_str(btype);
            }

            _ => {}
        }
    }

    fn format_stmt_type(&mut self, stmt: &StmtType) {
        match stmt {
            StmtType::Assign { lval, exp } => {
                self.format_node(lval);
                self.write_str(" = ");
                self.format_node(exp);
                self.write_str(";");
            }

            StmtType::Exp(exp_opt) => {
                if let Some(exp) = exp_opt {
                    self.format_node(exp);
                }
                self.write_str(";");
            }

            StmtType::Block(block) => {
                self.format_node(block);
            }

            StmtType::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                self.write_str("if (");
                self.format_node(cond);
                self.write_str(")");

                // 检查 then_stmt 是否是 Block
                if let AstNode::Stmt(inner_stmt) = then_stmt.as_ref() {
                    if let StmtType::Block(_) = inner_stmt.as_ref() {
                        self.write_str(" ");
                        self.format_node(then_stmt);
                    } else {
                        self.write_newline();
                        self.indent_level += 1;
                        self.write_indent();
                        self.format_node(then_stmt);
                        self.indent_level -= 1;
                    }
                } else {
                    self.write_newline();
                    self.indent_level += 1;
                    self.write_indent();
                    self.format_node(then_stmt);
                    self.indent_level -= 1;
                }

                if let Some(else_part) = else_stmt {
                    self.write_newline();
                    self.write_indent();
                    self.write_str("else");

                    if let AstNode::Stmt(else_inner) = else_part.as_ref() {
                        match else_inner.as_ref() {
                            StmtType::If { .. } => {
                                self.write_str(" ");
                                self.format_stmt_type(else_inner);
                            }
                            StmtType::Block(_) => {
                                self.write_str(" ");
                                self.format_node(else_part);
                            }
                            _ => {
                                self.write_newline();
                                self.indent_level += 1;
                                self.write_indent();
                                self.format_node(else_part);
                                self.indent_level -= 1;
                            }
                        }
                    } else {
                        self.write_newline();
                        self.indent_level += 1;
                        self.write_indent();
                        self.format_node(else_part);
                        self.indent_level -= 1;
                    }
                }
            }

            StmtType::While { cond, stmt } => {
                self.write_str("while (");
                self.format_node(cond);
                self.write_str(")");

                // 检查 stmt 是否是 Block
                if let AstNode::Stmt(inner_stmt) = stmt.as_ref() {
                    if let StmtType::Block(_) = inner_stmt.as_ref() {
                        self.write_str(" ");
                        self.format_node(stmt);
                    } else {
                        self.write_newline();
                        self.write_indent();
                        self.format_node(stmt);
                    }
                } else {
                    self.write_newline();
                    self.write_indent();
                    self.format_node(stmt);
                }
            }

            StmtType::Break => {
                self.write_str("break;");
            }

            StmtType::Continue => {
                self.write_str("continue;");
            }

            StmtType::Return(exp_opt) => {
                if let Some(exp) = exp_opt {
                    self.write_str("return ");
                    self.format_node(exp);
                    self.write_str(";");
                } else {
                    self.write_str("return;");
                }
            }
        }
    }
}

pub fn format_ast(ast: &AstNode) -> String {
    let mut formatter = Formatter::new();
    formatter.format_ast(ast)
}

#[test]
pub fn test_formatter() {
    // let src = std::fs::read_to_string("./tests/parser/sample1.in").unwrap_or_default();
    // let root = crate::parser::parse(&src).unwrap_or_else(|e| {
    //     println!("{}", e);
    //     panic!()
    // });
    // println!("{}", format_ast(&root));
    let case_dir = std::path::Path::new("./tests/parser");

    let mut entries: Vec<_> = fs::read_dir(case_dir)
        .unwrap_or_else(|_| panic!("Failed to read dir"))
        .map(|res| res.unwrap().path())
        .filter(|path| path.extension().map(|e| e == "in").unwrap_or(false))
        .collect();

    entries.sort();

    for entry in entries {
        let mut out_buffer = String::new();
        let input_path = entry.to_str().unwrap();
        let solution_path = input_path.trim_end_matches(".in").to_string() + ".out";

        let src = fs::read_to_string(input_path).expect("Failed to read file");
        out_buffer.push_str(&format!(
            "{}",
            match crate::parser::parse(&src, BuildConfig::default()) {
                Ok(root) => format_ast(&root),
                Err(e) => {
                    format!("{}", e)
                }
            }
        ));

        let solution = fs::read_to_string(solution_path).unwrap_or_else(|_| {
            panic!("Failed to read solution");
        });
        if out_buffer != solution {
            let mut diff_pos = 0;
            let out_chars: Vec<char> = out_buffer.chars().collect();
            let sol_chars: Vec<char> = solution.chars().collect();

            let max_len = out_chars.len().max(sol_chars.len());

            for i in 0..max_len {
                let out_char = out_chars.get(i).copied();
                let sol_char = sol_chars.get(i).copied();

                if out_char != sol_char {
                    diff_pos = i;
                    break;
                }
            }

            let format_char = |c: Option<char>| -> String {
                match c {
                    Some('\n') => "\\n".to_string(),
                    Some('\r') => "\\r".to_string(),
                    Some('\t') => "\\t".to_string(),
                    Some(' ') => "' '".to_string(),
                    Some(c) => format!("'{}'", c),
                    None => "EOF".to_string(),
                }
            };

            let out_char_at_pos = out_chars.get(diff_pos).copied();
            let sol_char_at_pos = sol_chars.get(diff_pos).copied();

            panic!(
                "Test failed for {}\nFirst difference at position {}: expected {}, found {}\n\nExpected:\n{}\n\nFound:\n{}",
                input_path,
                diff_pos,
                format_char(sol_char_at_pos),
                format_char(out_char_at_pos),
                solution,
                out_buffer
            );
        }
        println!(
            "{} passed.",
            input_path
                .rsplit('/')
                .next()
                .unwrap()
                .trim_end_matches(".in")
        );
    }
}

#[test]
pub fn test_single() {
    let src = std::fs::read_to_string("./tests/parser/hack2.in").unwrap_or_default();
    match crate::parser::parse(&src, BuildConfig::default()) {
        Ok(root) => println!("{}", format_ast(&root)),
        Err(e) => {
            println!("{}", e)
        }
    }
}
