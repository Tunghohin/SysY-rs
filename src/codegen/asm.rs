use crate::codegen::ir::LLVMIRGenerator;
use crate::codegen::regs::{
    LinearScanRegisterAllocator, Location, NoneRegisterAllocator, RV32IReg, RegisterAllocator,
};
use inkwell::builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{AsValueRef, BasicValue, FunctionValue, InstructionOpcode, InstructionValue};

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

    pub fn mul(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    mul {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn mulh(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    mulh {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn mulhsu(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    mulhsu {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn mulhu(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    mulhu {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn div(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    div {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn divu(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    divu {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn rem(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    rem {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn remu(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    remu {}, {}, {}\n", dest, src1, src2);
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

    pub fn la(&mut self, dest: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    la {}, {}\n", dest, label);
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
        self.buf += ".data\n";
        self
    }

    pub fn text_section(&mut self) -> &mut Self {
        self.buf += ".text\n";
        self
    }

    pub fn globl(&mut self, label: &str) -> &mut Self {
        self.buf += &format!(".globl {}\n", label);
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

pub struct RV32IASMGenerator<'ctx, T: RegisterAllocator> {
    ir_module: LLVMIRGenerator<'ctx>,
    allocator: T,
    builder: RV32IBuilder,
    entry: &'static str,
}

enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    SRem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Sar,
    Slt,
    Sltu,
}

impl<'ctx, T: RegisterAllocator> RV32IASMGenerator<'ctx, T> {
    pub fn new(ir_module: LLVMIRGenerator<'ctx>, entry: &'static str) -> Self {
        let mut ret = Self {
            ir_module,
            builder: RV32IBuilder::new(),
            allocator: T::new(),
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
            self.allocator.alloc_global(&global_var)?;
            self.builder
                .data_section()
                .tag(name)
                .word(init_val)
                .newline();
        }
        Ok(())
    }

    fn stack_size_align16(size: usize) -> usize {
        (size + 15) & !15
    }

    fn gen_prologue(&mut self, func: &FunctionValue<'_>) -> Result<(), String> {
        let stack_size = func.get_basic_block_iter().fold(0, |acc, bb| {
            acc + bb.get_instructions().fold(0, |acc2, intr| {
                if !intr.get_type().is_void_type() {
                    acc2 + 4
                } else {
                    acc2
                }
            })
        });
        self.builder
            .li(RV32IReg::T0, -(Self::stack_size_align16(stack_size) as i32))
            .add(RV32IReg::Sp, RV32IReg::Sp, RV32IReg::T0);
        Ok(())
    }

    fn gen_function(&mut self, func: &'_ FunctionValue<'_>) -> Result<(), String> {
        let name = func.get_name().to_str().map_err(|e| e.to_string())?;
        self.builder.text_section().globl(name).tag(name);

        self.gen_prologue(&func)?;
        self.allocator.alloc_in_function(&func)?;

        for bb in func.get_basic_blocks() {
            self.builder
                .tag(bb.get_name().to_str().map_err(|e| e.to_string())?);
            for instr in bb.get_instructions() {
                match instr.get_opcode() {
                    InstructionOpcode::Return => self.gen_return(&instr, name == self.entry)?,
                    InstructionOpcode::Load => self.gen_load(&instr)?,
                    InstructionOpcode::Store => self.gen_store(&instr)?,
                    InstructionOpcode::Add => self.gen_binary(&instr, Operator::Add)?,
                    InstructionOpcode::Sub => self.gen_binary(&instr, Operator::Sub)?,
                    InstructionOpcode::Mul => self.gen_binary(&instr, Operator::Mul)?,
                    InstructionOpcode::SDiv => self.gen_binary(&instr, Operator::Div)?,
                    InstructionOpcode::SRem => self.gen_binary(&instr, Operator::SRem)?,
                    // Add more binary operators here as needed
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn gen_load(&mut self, instr: &InstructionValue<'_>) -> Result<(), String> {
        if instr.get_num_operands() != 1 {
            return Err("Load instruction must have 1 operand".to_string());
        }

        let src = instr
            .get_operand(0)
            .ok_or("Load instruction missing source operand")?
            .left()
            .ok_or("Invalid source operand")?
            .as_value_ref();
        let dest = instr.as_value_ref();

        let dest_loc = self
            .allocator
            .get(&dest)
            .ok_or("Destination location not found")?;
        let src_loc = self
            .allocator
            .get(&src)
            .ok_or("Source location not found")?;

        match src_loc {
            Location::Global(name) => {
                self.builder.la(RV32IReg::T0, &name);
                self.builder.lw(RV32IReg::T0, 0, RV32IReg::T0);
            }
            Location::Stack(offset) => {
                self.builder.lw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            _ => {
                return Err("Unsupported source location for load".to_string());
            }
        }

        match dest_loc {
            Location::Reg(reg) => {
                self.builder.mv(reg, RV32IReg::T0);
            }
            Location::Stack(offset) => {
                self.builder.sw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            _ => {
                return Err("Unsupported destination location for load".to_string());
            }
        }

        Ok(())
    }

    fn gen_store(&mut self, instr: &InstructionValue<'_>) -> Result<(), String> {
        if instr.get_num_operands() != 2 {
            return Err("Store instruction must have 2 operands".to_string());
        }

        let value = instr
            .get_operand(0)
            .ok_or("Store instruction missing value operand")?
            .left()
            .ok_or("Invalid value operand")?
            .as_value_ref();
        let address = instr
            .get_operand(1)
            .ok_or("Store instruction missing address operand")?
            .left()
            .ok_or("Invalid address operand")?
            .as_value_ref();

        let value_loc = self.allocator.get(&value);
        let address_loc = self.allocator.get(&address);

        match value_loc {
            Some(loc) => match loc {
                Location::Reg(reg) => {
                    self.builder.mv(RV32IReg::T0, reg);
                }
                Location::Stack(offset) => {
                    self.builder.lw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
                }
                Location::Global(name) => {
                    self.builder.la(RV32IReg::T0, &name);
                    self.builder.lw(RV32IReg::T0, 0, RV32IReg::T0);
                }
            },
            None => {
                self.builder.li(
                    RV32IReg::T0,
                    instr
                        .get_operand(0)
                        .ok_or("Missing value operand")?
                        .left()
                        .ok_or("Invalid value operand")?
                        .into_int_value()
                        .get_zero_extended_constant()
                        .ok_or("Value is not a constant")? as i32,
                );
            }
        }

        match address_loc {
            Some(loc) => match loc {
                Location::Reg(reg) => {
                    self.builder.sw(RV32IReg::T0, 0, reg);
                }
                Location::Stack(offset) => {
                    self.builder.sw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
                }
                Location::Global(name) => {
                    self.builder.la(RV32IReg::T1, &name);
                    self.builder.sw(RV32IReg::T0, 0, RV32IReg::T1);
                }
            },
            None => {
                self.builder.li(
                    RV32IReg::T1,
                    instr
                        .get_operand(1)
                        .ok_or("Missing address operand")?
                        .left()
                        .ok_or("Invalid address operand")?
                        .into_int_value()
                        .get_zero_extended_constant()
                        .ok_or("Address is not a constant")? as i32,
                );
                self.builder.sw(RV32IReg::T0, 0, RV32IReg::T1);
            }
        }

        Ok(())
    }

    fn gen_binary(&mut self, instr: &InstructionValue<'_>, op: Operator) -> Result<(), String> {
        if instr.get_num_operands() != 2 {
            return Err("Binary instruction must have 2 operands".to_string());
        }

        let lhs = instr
            .get_operand(0)
            .ok_or("Binary instruction missing lhs operand")?
            .left()
            .ok_or("Invalid lhs operand")?
            .as_value_ref();
        let rhs = instr
            .get_operand(1)
            .ok_or("Binary instruction missing rhs operand")?
            .left()
            .ok_or("Invalid rhs operand")?
            .as_value_ref();
        let dest = instr.as_value_ref();

        let dest_loc = self
            .allocator
            .get(&dest)
            .ok_or("Destination location not found")?;
        let lhs_loc = self.allocator.get(&lhs);
        let rhs_loc = self.allocator.get(&rhs);

        match lhs_loc {
            Some(loc) => match loc {
                Location::Reg(reg) => {
                    self.builder.mv(RV32IReg::T0, reg);
                }
                Location::Stack(offset) => {
                    self.builder.lw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
                }
                Location::Global(name) => {
                    self.builder.la(RV32IReg::T0, &name);
                    self.builder.lw(RV32IReg::T0, 0, RV32IReg::T0);
                }
            },
            None => {
                self.builder.li(
                    RV32IReg::T0,
                    instr
                        .get_operand(0)
                        .ok_or("Missing lhs operand")?
                        .left()
                        .ok_or("Invalid lhs operand")?
                        .into_int_value()
                        .get_zero_extended_constant()
                        .ok_or("LHS is not a constant")? as i32,
                );
            }
        }
        match rhs_loc {
            Some(loc) => match loc {
                Location::Reg(reg) => {
                    self.builder.mv(RV32IReg::T1, reg);
                }
                Location::Stack(offset) => {
                    self.builder.lw(RV32IReg::T1, offset as i32, RV32IReg::Sp);
                }
                Location::Global(label) => {
                    self.builder.la(RV32IReg::T1, &label);
                    self.builder.lw(RV32IReg::T1, 0, RV32IReg::T1);
                }
            },
            None => {
                self.builder.li(
                    RV32IReg::T1,
                    instr
                        .get_operand(1)
                        .ok_or("Missing rhs operand")?
                        .left()
                        .ok_or("Invalid rhs operand")?
                        .into_int_value()
                        .get_zero_extended_constant()
                        .ok_or("RHS is not a constant")? as i32,
                );
            }
        }

        match op {
            Operator::Add => {
                self.builder.add(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            Operator::Sub => {
                self.builder.sub(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            Operator::Mul => {
                self.builder.mul(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            Operator::Div => {
                self.builder.div(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            Operator::SRem => {
                self.builder.rem(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            _ => {
                return Err("Unsupported binary operator".to_string());
            }
        }

        match dest_loc {
            Location::Reg(reg) => {
                self.builder.mv(reg, RV32IReg::T0);
            }
            Location::Stack(offset) => {
                self.builder.sw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            Location::Global(label) => {
                self.builder.la(RV32IReg::T1, &label);
                self.builder.sw(RV32IReg::T0, 0, RV32IReg::T1);
            }
        }

        Ok(())
    }

    fn gen_return(&mut self, instr: &InstructionValue<'_>, is_entry: bool) -> Result<(), String> {
        let num_operands = instr.get_num_operands();

        if (num_operands == 0) {
            self.builder.ret();
            return Ok(());
        }

        let ret_val = instr
            .get_operand(0)
            .ok_or("Return instruction missing operand")?;

        let val = ret_val
            .left()
            .ok_or("Invalid return operand")?
            .as_value_ref();
        match self.allocator.get(&val) {
            Some(loc) => match loc {
                Location::Reg(reg) => {
                    self.builder.mv(RV32IReg::A0, reg);
                }
                Location::Stack(offset) => {
                    self.builder.lw(RV32IReg::A0, offset as i32, RV32IReg::Sp);
                }
                Location::Global(name) => {
                    self.builder.la(RV32IReg::T0, &name);
                    self.builder.lw(RV32IReg::A0, 0, RV32IReg::T0);
                }
            },
            None => {
                self.builder.li(
                    RV32IReg::A0,
                    ret_val
                        .left()
                        .ok_or("Invalid return operand")?
                        .into_int_value()
                        .get_zero_extended_constant()
                        .ok_or("Return value is not a constant")? as i32,
                );
            }
        }
        self.builder
            .li(RV32IReg::T0, 256)
            .rem(RV32IReg::A0, RV32IReg::A0, RV32IReg::T0);

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

    pub fn emit(&self) -> &str {
        self.builder.emit()
    }
}
