use crate::codegen::ir::LLVMIRGenerator;
use crate::codegen::regs::RV32IReg;
use inkwell::builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, InstructionOpcode, InstructionValue};

#[derive(Clone, Copy, Debug)]
pub struct VarId(pub u32);

struct RV32IBuilder {
    buf: String,
}

impl RV32IBuilder {
    fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn addi(&mut self, dest: RV32IReg, src: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    addi {}, {}, 0x{:x}\n", dest, src, imm);
        self
    }

    pub fn lui(&mut self, dest: &str, imm: i32) -> &mut Self {
        self.buf += &format!("    lui {}, 0x{:x}\n", dest, imm);
        self
    }

    pub fn jal(&mut self, dest: &str, offset: i32) -> &mut Self {
        self.buf += &format!("    jal {}, 0x{:x}\n", dest, offset);
        self
    }

    pub fn beq(&mut self, src1: &str, src2: &str, offset: i32) -> &mut Self {
        self.buf += &format!("    beq {}, {}, 0x{:x}\n", src1, src2, offset);
        self
    }

    pub fn add(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    add {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sub(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    sub {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn and(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    and {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn or(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    or {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn xor(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    xor {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sll(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    sll {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn srl(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    srl {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sra(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    sra {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn slt(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    slt {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sltu(&mut self, dest: &str, src1: &str, src2: &str) -> &mut Self {
        self.buf += &format!("    sltu {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn lb(&mut self, dest: &str, offset: i32, base: &str) -> &mut Self {
        self.buf += &format!("    lb {}, {}({})\n", dest, offset, base);
        self
    }

    pub fn lh(&mut self, dest: &str, offset: i32, base: &str) -> &mut Self {
        self.buf += &format!("    lh {}, {}({})\n", dest, offset, base);
        self
    }

    pub fn lw(&mut self, dest: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    lw {}, {}({})\n", dest, offset, base);
        self
    }

    pub fn sb(&mut self, src: &str, offset: i32, base: &str) -> &mut Self {
        self.buf += &format!("    sb {}, {}({})\n", src, offset, base);
        self
    }

    pub fn sh(&mut self, src: &str, offset: i32, base: &str) -> &mut Self {
        self.buf += &format!("    sh {}, {}({})\n", src, offset, base);
        self
    }

    pub fn sw(&mut self, src: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    sw {}, {}({})\n", src, offset, base);
        self
    }

    pub fn li(&mut self, dest: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    li {}, 0x{:x}\n", dest, imm);
        self
    }

    pub fn mv(&mut self, dest: RV32IReg, src: RV32IReg) -> &mut Self {
        self.addi(dest, src, 0)
    }

    pub fn nop(&mut self) -> &mut Self {
        self.addi(RV32IReg::Zero, RV32IReg::Zero, 0)
    }

    pub fn ret(&mut self) -> &mut Self {
        self.buf += "    jalr x0, x1, 0\n";
        self
    }

    pub fn call(&mut self, label: &str) -> &mut Self {
        self.buf += &format!("    jal x1, {}\n", label);
        self
    }

    pub fn tail(&mut self, label: &str) -> &mut Self {
        self.buf += &format!("    jal x0, {}\n", label);
        self
    }

    pub fn ecall(&mut self) -> &mut Self {
        self.buf += "    ecall\n";
        self
    }

    pub fn tag(&mut self, tag: &str) -> &mut Self {
        self.buf += &format!("{}:\n", tag);
        self
    }

    pub fn word(&mut self, value: i32) -> &mut Self {
        self.buf += &format!("    .word {}\n", value);
        self
    }

    pub fn data_section(&mut self) -> &mut Self {
        self.buf += "    .data\n";
        self
    }

    pub fn text_section(&mut self) -> &mut Self {
        self.buf += "    .text\n";
        self
    }

    pub fn globl(&mut self, label: &str) -> &mut Self {
        self.buf += &format!("    .globl {}\n", label);
        self
    }

    pub fn emit(&self) -> &str {
        &self.buf
    }

    pub fn newline(&mut self) -> &mut Self {
        self.buf += "\n";
        self
    }
}

trait RegisterAllocator {
    fn alloc(&mut self) -> Result<RV32IReg, String>;
    fn free(&mut self, reg: RV32IReg);
}

struct LinearScanRegisterAllocator {
    available_tmps: Vec<RV32IReg>,
}

impl LinearScanRegisterAllocator {
    pub fn new() -> Self {
        Self {
            available_tmps: vec![
                RV32IReg::T0,
                RV32IReg::T1,
                RV32IReg::T2,
                RV32IReg::T3,
                RV32IReg::T4,
                RV32IReg::T5,
                RV32IReg::T6,
            ],
        }
    }
}

impl RegisterAllocator for LinearScanRegisterAllocator {
    fn alloc(&mut self) -> Result<RV32IReg, String> {
        self.available_tmps
            .pop()
            .ok_or_else(|| "No available registers".to_string())
    }

    fn free(&mut self, reg: RV32IReg) {
        self.available_tmps.push(reg);
    }
}

pub struct RV32IASMGenerator<'ctx> {
    ir_module: LLVMIRGenerator<'ctx>,
    builder: RV32IBuilder,
    reg_allocator: LinearScanRegisterAllocator,
    entry: &'static str,
}

impl<'ctx> RV32IASMGenerator<'ctx> {
    pub fn new(ir_module: LLVMIRGenerator<'ctx>, entry: &'static str) -> Self {
        let mut ret = Self {
            ir_module,
            reg_allocator: LinearScanRegisterAllocator::new(),
            builder: RV32IBuilder::new(),
            entry,
        };
        ret.ir_module.optimize();
        ret
    }

    fn gen_global_vars(&mut self) -> Result<(), String> {
        for global_var in self.ir_module.module().get_globals() {
            let name = global_var.get_name().to_str().map_err(|e| e.to_string())?;
            let init_val = global_var
                .get_initializer()
                .unwrap()
                .into_int_value()
                .get_zero_extended_constant()
                .unwrap() as i32;
            self.builder
                .data_section()
                .globl(name)
                .tag(name)
                .word(init_val)
                .newline();
        }
        Ok(())
    }

    fn stack_size_align16(size: usize) -> usize {
        (size + 15) & !15
    }

    fn gen_prologue(&mut self, func: &FunctionValue<'ctx>) -> Result<(), String> {
        let stack_size = func.get_basic_block_iter().fold(0, |acc, bb| {
            acc + bb.get_instructions().fold(0, |acc2, intr| {
                println!("Instruction: {:?}", intr);
                match intr.get_opcode() {
                    _ => 4,
                }
            })
        });
        self.builder.addi(
            RV32IReg::Sp,
            RV32IReg::Sp,
            -(Self::stack_size_align16(stack_size) as i32),
        );
        Ok(())
    }

    fn gen_function(&mut self, func: &FunctionValue<'ctx>) -> Result<(), String> {
        let name = func.get_name().to_str().map_err(|e| e.to_string())?;
        self.builder.text_section().globl(name).tag(name);

        self.gen_prologue(&func)?;

        for bb in func.get_basic_blocks() {
            self.builder
                .tag(bb.get_name().to_str().map_err(|e| e.to_string())?);
            for instr in bb.get_instructions() {
                match instr.get_opcode() {
                    InstructionOpcode::Return => self.gen_return(&instr, name == self.entry)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn gen_return(
        &mut self,
        instr: &inkwell::values::InstructionValue<'ctx>,
        is_entry: bool,
    ) -> Result<(), String> {
        let num_operands = instr.get_num_operands();

        if num_operands == 0 {
            self.builder.ret();
            return Ok(());
        }

        let ret_val = instr
            .get_operand(0)
            .ok_or("Return instruction missing operand")?;

        let val = ret_val.left().ok_or("Invalid return operand")?;

        if val.is_int_value() {
            let imm = val
                .into_int_value()
                .get_zero_extended_constant()
                .ok_or("Failed to get return imm")? as i32;
            self.builder.li(RV32IReg::A0, imm);
        } else {
            // unimplemented!()
        }

        if is_entry {
            self.builder.li(RV32IReg::A7, 93).ecall();
        } else {
            self.builder.ret();
        }

        Ok(())
    }

    fn gen_functions(&mut self) -> Result<(), String> {
        for func in self.ir_module.module().get_functions() {
            self.gen_function(&func)?;
        }
        Ok(())
    }

    pub fn gen_asm(&mut self) -> Result<(), String> {
        self.gen_global_vars()?;
        self.gen_functions()?;
        Ok(())
    }

    pub fn print_to_stderr(&self) {
        eprintln!("{}", self.builder.emit());
    }
}
