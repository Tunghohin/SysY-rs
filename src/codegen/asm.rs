use crate::codegen::regs::RV32IReg;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, InstructionOpcode};

#[derive(Clone, Copy, Debug)]
pub struct VarId(pub u32);

#[derive(Clone, Debug)]
pub enum Location {
    Reg(&'static str),
    Stack(i32),
    Global(&'static str),
}

pub trait RegisterAllocator {
    fn allocate(&mut self, v: VarId) -> Location;
    fn stack_size(&self) -> usize;
}

pub struct NoAlloc;
impl RegisterAllocator for NoAlloc {
    fn allocate(&mut self, _v: VarId) -> Location {
        Location::Stack(0)
    }
    fn stack_size(&self) -> usize {
        64 * 1024
    }
}

pub struct LinearScan {}
impl RegisterAllocator for LinearScan {
    fn allocate(&mut self, v: VarId) -> Location {
        Location::Reg("t0")
    }
    fn stack_size(&self) -> usize {
        64
    }
}

struct RV32IBuilder {
    buf: String,
}

impl RV32IBuilder {
    fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn addi(&mut self, dest: RV32IReg, src: RV32IReg, imm: i32) {
        self.buf += &format!("    addi {}, {}, 0x{:x}\n", dest, src, imm);
    }

    pub fn lui(&mut self, dest: &str, imm: i32) {
        self.buf += &format!("    lui {}, 0x{:x}\n", dest, imm);
    }

    pub fn jal(&mut self, dest: &str, offset: i32) {
        self.buf += &format!("    jal {}, 0x{:x}\n", dest, offset);
    }

    pub fn beq(&mut self, src1: &str, src2: &str, offset: i32) {
        self.buf += &format!("    beq {}, {}, 0x{:x}\n", src1, src2, offset);
    }

    pub fn add(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) {
        self.buf += &format!("    add {}, {}, {}\n", dest, src1, src2);
    }

    pub fn sub(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) {
        self.buf += &format!("    sub {}, {}, {}\n", dest, src1, src2);
    }

    pub fn and(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    and {}, {}, {}\n", dest, src1, src2);
    }

    pub fn or(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    or {}, {}, {}\n", dest, src1, src2);
    }

    pub fn xor(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    xor {}, {}, {}\n", dest, src1, src2);
    }

    pub fn sll(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    sll {}, {}, {}\n", dest, src1, src2);
    }

    pub fn srl(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    srl {}, {}, {}\n", dest, src1, src2);
    }

    pub fn sra(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    sra {}, {}, {}\n", dest, src1, src2);
    }

    pub fn slt(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    slt {}, {}, {}\n", dest, src1, src2);
    }

    pub fn sltu(&mut self, dest: &str, src1: &str, src2: &str) {
        self.buf += &format!("    sltu {}, {}, {}\n", dest, src1, src2);
    }

    pub fn lb(&mut self, dest: &str, offset: i32, base: &str) {
        self.buf += &format!("    lb {}, {}({})\n", dest, offset, base);
    }

    pub fn lh(&mut self, dest: &str, offset: i32, base: &str) {
        self.buf += &format!("    lh {}, {}({})\n", dest, offset, base);
    }

    pub fn lw(&mut self, dest: RV32IReg, offset: i32, base: RV32IReg) {
        self.buf += &format!("    lw {}, {}({})\n", dest, offset, base);
    }

    pub fn sb(&mut self, src: &str, offset: i32, base: &str) {
        self.buf += &format!("    sb {}, {}({})\n", src, offset, base);
    }

    pub fn sh(&mut self, src: &str, offset: i32, base: &str) {
        self.buf += &format!("    sh {}, {}({})\n", src, offset, base);
    }

    pub fn sw(&mut self, src: RV32IReg, offset: i32, base: RV32IReg) {
        self.buf += &format!("    sw {}, {}({})\n", src, offset, base);
    }

    pub fn li(&mut self, dest: RV32IReg, imm: i32) {
        if imm >= -(1 << 11) && imm < (1 << 11) {
            self.addi(dest, RV32IReg::Zero, imm);
        } else {
            let upper = (imm + (1 << 11)) >> 12;
            let lower = imm & 0xfff;
            self.lui(&format!("{}", dest), upper);
            self.addi(dest.clone(), dest, lower);
        }
    }

    pub fn mv(&mut self, dest: RV32IReg, src: RV32IReg) {
        self.addi(dest, src, 0);
    }

    pub fn nop(&mut self) {
        self.addi(RV32IReg::Zero, RV32IReg::Zero, 0);
    }

    pub fn ret(&mut self) {
        self.buf += "    jalr x0, x1, 0\n";
    }

    pub fn call(&mut self, label: &str) {
        self.buf += &format!("    jal x1, {}\n", label);
    }

    pub fn tail(&mut self, label: &str) {
        self.buf += &format!("    jal x0, {}\n", label);
    }

    pub fn ecall(&mut self) {
        self.buf += "    ecall\n";
    }

    pub fn emit(&self) -> &str {
        &self.buf
    }
}

pub fn emit_rv32i_asm(
    module: &Module,
    reg_alloc: &mut dyn RegisterAllocator,
    entry_point: &str,
) -> String {
    let mut builder = RV32IBuilder::new();

    for func in module.get_functions() {
        for bb in func.get_basic_blocks() {
            for inst in bb.get_instructions() {
                unimplemented!()
            }
            if func.get_name().to_str().unwrap() == entry_point {
                builder.li(RV32IReg::A7, 93); // syscall number for exit
                builder.ecall();
            } else {
                builder.ret();
            }
        }
    }

    builder.emit().to_string()
}
