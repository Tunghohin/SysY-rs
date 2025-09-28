mod utils;

use std::collections::HashMap;
use syn::{
    Block, Expr, ExprBinary, File, FnArg, ItemFn, Lit, Local, ReturnType, Stmt, Type,
    punctuated::Punctuated,
};

struct MIREmitter {
    syntax: File,
    indent: usize,
    temp_cnt: usize,
    bbn_cnt: usize,
    var_map: HashMap<String, usize>,
    args: Vec<String>,
}

impl MIREmitter {
    fn new(syntax: File) -> Self {
        MIREmitter {
            syntax,
            indent: 0,
            temp_cnt: 1, // 0 for return
            bbn_cnt: 0,
            var_map: HashMap::new(),
            args: vec![],
        }
    }

    fn inc_indent(&mut self) {
        self.indent += 1;
    }

    fn dec_indent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn inc_bbn(&mut self) -> usize {
        let cnt = self.bbn_cnt;
        self.bbn_cnt += 1;
        cnt
    }

    fn entry_bbn(&mut self, id: usize) {
        utils::write(&format!("bbn{}:", id), self.indent);
        self.inc_indent();
    }

    fn exit_bbn(&mut self) {
        self.dec_indent();
        utils::writeln("}", self.indent);
    }

    fn inc_var_temp(&mut self) -> usize {
        let cnt = self.temp_cnt;
        self.temp_cnt += 1;
        cnt
    }

    fn emit(&mut self) {
        let items = self.syntax.items.clone();
        for input in items {
            if let syn::Item::Fn(func) = input {
                self.emit_func(&func);
            }
        }
    }

    fn emit_func(&mut self, func: &ItemFn) {
        utils::write(&format!("fn {}(", func.sig.ident), self.indent);
        self.emit_func_args(&func.sig.inputs);
        utils::write(") ", 0);
        match &func.sig.output {
            ReturnType::Default => {
                utils::write("-> () ", 0);
            }
            ReturnType::Type(_, ty) => {
                if let Type::Path(type_path) = &**ty {
                    let type_name = type_path.path.segments.last().unwrap().ident.to_string();
                    utils::write(&format!("-> {} ", type_name), 0);
                }
            }
        }
        utils::writeln("{", 0);
        self.inc_indent();

        self.emit_debug_vars();

        for stmt in &func.block.stmts {
            self.emit_stmt(stmt);
        }

        self.dec_indent();
        utils::write("}", self.indent)
    }

    fn emit_debug_vars(&mut self) {
        for arg in self.args.iter() {
            utils::writeln(
                &format!("debug {} => _{};", arg, self.var_map.get(arg).unwrap()),
                self.indent,
            );
        }
    }

    fn emit_func_args(&mut self, args: &Punctuated<FnArg, syn::token::Comma>) {
        for (idx, arg) in args.iter().enumerate() {
            if idx != 0 {
                utils::write(", ", 0);
            }
            if let FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let var_name = pat_ident.ident.to_string();
                    let temp_idx = self.inc_var_temp();
                    self.var_map.insert(var_name.clone(), temp_idx);
                    self.args.push(var_name.clone());
                    utils::write(&format!("_{}", temp_idx), 0);
                }
                if let Type::Path(type_path) = &*pat_type.ty {
                    let type_name = type_path.path.segments.last().unwrap().ident.to_string();
                    utils::write(&format!(": {}", type_name), 0);
                }
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign(_) => {}
            Expr::Binary(binary_expr) => {}
            Expr::Lit(lit) => {}
            _ => {}
        }
    }

    fn emit_local(&mut self, local: &Local) {}

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Local(local) => self.emit_local(local),
            Stmt::Expr(expr, _) => self.emit_expr(expr),
            _ => unimplemented!(),
        }
    }
}

pub fn gen_mir(syntax: &File) {
    // for input in &syntax.items {
    //     if let syn::Item::Fn(func) = input {
    //         let mut emitter = MIREmitter::new(syntax.clone());
    //         emitter.emit();
    //     }
    // }
    println!("{:#?}", syntax)
}
