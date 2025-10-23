use std::num;

use crate::codegen::ir::LLVMIRGenerator;
use crate::codegen::regs::{Location, NoneRegisterAllocator, RV32IReg, RegisterAllocator};
use inkwell::builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{AsValueRef, BasicValue, FunctionValue, InstructionOpcode, InstructionValue};

#[derive(Clone, Copy, Debug)]
pub struct VarId(pub u32);

pub struct RV32IBuilder {
    buf: String,
}

impl RV32IBuilder {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn addi(&mut self, dest: RV32IReg, src: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    addi {}, {}, {}\n", dest, src, imm);
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

    pub fn xor(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    xor {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn and(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    and {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn or(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    or {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sll(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    sll {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn srl(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    srl {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sra(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    sra {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn slt(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    slt {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn sltu(&mut self, dest: RV32IReg, src1: RV32IReg, src2: RV32IReg) -> &mut Self {
        self.buf += &format!("    sltu {}, {}, {}\n", dest, src1, src2);
        self
    }

    pub fn slti(&mut self, dest: RV32IReg, src: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    slti {}, {}, {}\n", dest, src, imm);
        self
    }

    pub fn sltiu(&mut self, dest: RV32IReg, src: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    sltiu {}, {}, {}\n", dest, src, imm);
        self
    }

    pub fn lui(&mut self, dest: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    lui {}, {}\n", dest, imm);
        self
    }

    pub fn auipc(&mut self, dest: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    auipc {}, {}\n", dest, imm);
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

    pub fn seqz(&mut self, dest: RV32IReg, src: RV32IReg) -> &mut Self {
        self.buf += &format!("    seqz {}, {}\n", dest, src);
        self
    }

    pub fn snez(&mut self, dest: RV32IReg, src: RV32IReg) -> &mut Self {
        self.buf += &format!("    snez {}, {}\n", dest, src);
        self
    }

    pub fn sltz(&mut self, dest: RV32IReg, src: RV32IReg) -> &mut Self {
        self.buf += &format!("    sltz {}, {}\n", dest, src);
        self
    }

    pub fn sgtz(&mut self, dest: RV32IReg, src: RV32IReg) -> &mut Self {
        self.buf += &format!("    sgtz {}, {}\n", dest, src);
        self
    }

    pub fn beq(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    beq {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn bne(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    bne {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn blt(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    blt {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn bge(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    bge {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn bltu(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    bltu {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn bgeu(&mut self, src1: RV32IReg, src2: RV32IReg, label: &str) -> &mut Self {
        self.buf += &format!("    bgeu {}, {}, {}\n", src1, src2, label);
        self
    }

    pub fn lw(&mut self, dest: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    lw {}, {}({})\n", dest, offset, base);
        self
    }

    pub fn sw(&mut self, src: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    sw {}, {}({})\n", src, offset, base);
        self
    }

    pub fn lb(&mut self, dest: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    lb {}, {}({})\n", dest, offset, base);
        self
    }

    pub fn sb(&mut self, src: RV32IReg, offset: i32, base: RV32IReg) -> &mut Self {
        self.buf += &format!("    sb {}, {}({})\n", src, offset, base);
        self
    }

    pub fn li(&mut self, dest: RV32IReg, imm: i32) -> &mut Self {
        self.buf += &format!("    li {}, {}\n", dest, imm);
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

    pub fn newline(&mut self) -> &mut Self {
        self.buf += "\n";
        self
    }

    pub fn emit(&self) -> &str {
        &self.buf
    }
}

pub struct RV32IASMGenerator<'ctx, T: RegisterAllocator> {
    ir_module: LLVMIRGenerator<'ctx>,
    allocator: T,
    builder: RV32IBuilder,
    entry: &'static str,
}

enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    SRem,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Xor,
    Ashr,
    Lshr,
    Shl,
}

impl<'ctx, T: RegisterAllocator> RV32IASMGenerator<'ctx, T> {
    pub fn new(ir_module: LLVMIRGenerator<'ctx>, entry: &'static str) -> Self {
        let allocator = T::new(&ir_module.module());
        let mut ret = Self {
            ir_module,
            builder: RV32IBuilder::new(),
            allocator: allocator,
            entry,
        };
        ret.ir_module.optimize();
        ret
    }

    fn gen_global_vars(&mut self) -> Result<(), String> {
        for global_var in self.ir_module.module().get_globals() {
            self.builder
                .data_section()
                .tag(global_var.get_name().to_str().map_err(|e| e.to_string())?)
                .word(
                    global_var
                        .get_initializer()
                        .unwrap()
                        .into_int_value()
                        .get_zero_extended_constant()
                        .unwrap() as i32,
                )
                .newline();
        }
        Ok(())
    }

    fn stack_size_align16(size: usize) -> usize {
        (size + 15) & !15
    }

    fn gen_prologue(&mut self, stack_size: usize) -> Result<(), String> {
        self.builder.li(RV32IReg::T0, -(stack_size as i32)).add(
            RV32IReg::Sp,
            RV32IReg::Sp,
            RV32IReg::T0,
        );
        Ok(())
    }

    fn gen_function(&mut self, func: &'_ FunctionValue<'_>) -> Result<(), String> {
        let name = func.get_name().to_str().map_err(|e| e.to_string())?;
        self.builder.text_section().globl(name).tag(name);
        self.allocator.alloc(func)?;
        self.gen_prologue(self.allocator.stack_size_required())?;

        for bb in func.get_basic_blocks() {
            self.builder
                .tag(bb.get_name().to_str().map_err(|e| e.to_string())?);
            for instr in bb.get_instructions() {
                match instr.get_opcode() {
                    InstructionOpcode::Return => self.gen_return(&instr, name == self.entry)?,
                    InstructionOpcode::Load => self.gen_load(&instr)?,
                    InstructionOpcode::Store => self.gen_store(&instr)?,
                    InstructionOpcode::Add => self.gen_binary(&instr, BinaryOperator::Add)?,
                    InstructionOpcode::Sub => self.gen_binary(&instr, BinaryOperator::Sub)?,
                    InstructionOpcode::Mul => self.gen_binary(&instr, BinaryOperator::Mul)?,
                    InstructionOpcode::SDiv => self.gen_binary(&instr, BinaryOperator::Div)?,
                    InstructionOpcode::SRem => self.gen_binary(&instr, BinaryOperator::SRem)?,
                    InstructionOpcode::And => self.gen_binary(&instr, BinaryOperator::And)?,
                    InstructionOpcode::Or => self.gen_binary(&instr, BinaryOperator::Or)?,
                    InstructionOpcode::Xor => self.gen_binary(&instr, BinaryOperator::Xor)?,
                    InstructionOpcode::AShr => self.gen_binary(&instr, BinaryOperator::Ashr)?,
                    InstructionOpcode::LShr => self.gen_binary(&instr, BinaryOperator::Lshr)?,
                    InstructionOpcode::Shl => self.gen_binary(&instr, BinaryOperator::Shl)?,
                    InstructionOpcode::ICmp => self.gen_icmp(&instr)?,
                    InstructionOpcode::ZExt => self.gen_ext(&instr, false)?,
                    InstructionOpcode::SExt => self.gen_ext(&instr, true)?,
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn gen_ext(&mut self, instr: &InstructionValue<'_>, is_signed: bool) -> Result<(), String> {
        if instr.get_num_operands() != 1 {
            return Err("ZExt instruction must have 1 operand".to_string());
        }

        let dest_loc = self
            .allocator
            .get(&instr.as_value_ref())
            .ok_or("Destination location not found")?;
        let src_loc = self.allocator.get(
            &instr
                .get_operand(0)
                .ok_or("ZExt instruction missing source operand")?
                .left()
                .ok_or("Invalid source operand")?
                .as_value_ref(),
        );

        match src_loc {
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
                    if is_signed {
                        instr
                            .get_operand(0)
                            .ok_or("Missing source operand")?
                            .left()
                            .ok_or("Invalid source operand")?
                            .into_int_value()
                            .get_sign_extended_constant()
                            .ok_or("Source is not a constant")? as i32
                    } else {
                        instr
                            .get_operand(0)
                            .ok_or("Missing source operand")?
                            .left()
                            .ok_or("Invalid source operand")?
                            .into_int_value()
                            .get_zero_extended_constant()
                            .ok_or("Source is not a constant")? as i32
                    },
                );
            }
        }

        match dest_loc {
            Location::Reg(reg) => {
                self.builder.mv(reg, RV32IReg::T0);
            }
            Location::Stack(offset) => {
                self.builder.sw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            Location::Global(name) => {
                self.builder.la(RV32IReg::T1, &name);
                self.builder.sw(RV32IReg::T0, 0, RV32IReg::T1);
            }
        }

        Ok(())
    }

    fn gen_load(&mut self, instr: &InstructionValue<'_>) -> Result<(), String> {
        if instr.get_num_operands() != 1 {
            return Err("Load instruction must have 1 operand".to_string());
        }
        let dest_loc = self
            .allocator
            .get(&instr.as_value_ref())
            .ok_or("Destination location not found")?;
        let src_loc = self
            .allocator
            .get(
                &&instr
                    .get_operand(0)
                    .ok_or("Load instruction missing source operand")?
                    .left()
                    .ok_or("Invalid source operand")?
                    .as_value_ref(),
            )
            .ok_or("Source location not found")?;

        match src_loc {
            Location::Reg(reg) => {
                self.builder.lw(RV32IReg::T0, 0, reg);
            }
            Location::Stack(offset) => {
                self.builder.lw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            Location::Global(name) => {
                self.builder
                    .la(RV32IReg::T0, &name)
                    .lw(RV32IReg::T0, 0, RV32IReg::T0);
            }
        }

        match dest_loc {
            Location::Reg(reg) => {
                self.builder.mv(reg, RV32IReg::T0);
            }
            Location::Stack(offset) => {
                self.builder.sw(RV32IReg::T0, offset as i32, RV32IReg::Sp);
            }
            Location::Global(name) => {
                self.builder.la(RV32IReg::T1, &name);
                self.builder.sw(RV32IReg::T0, 0, RV32IReg::T1);
            }
        }

        Ok(())
    }

    fn gen_store(&mut self, instr: &InstructionValue<'_>) -> Result<(), String> {
        if instr.get_num_operands() != 2 {
            return Err("Store instruction must have 2 operands".to_string());
        }

        let src_loc = self.allocator.get(
            &instr
                .get_operand(0)
                .ok_or("Store instruction missing value operand")?
                .left()
                .ok_or("Invalid value operand")?
                .as_value_ref(),
        );
        let dst_loc = self
            .allocator
            .get(
                &instr
                    .get_operand(1)
                    .ok_or("Store instruction missing address operand")?
                    .left()
                    .ok_or("Invalid address operand")?
                    .as_value_ref(),
            )
            .unwrap();

        match src_loc {
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

        match dst_loc {
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
        }

        Ok(())
    }

    fn gen_icmp(&mut self, instr: &InstructionValue<'_>) -> Result<(), String> {
        if instr.get_num_operands() != 2 {
            return Err("ICmp instruction must have 3 operands".to_string());
        }

        let predicate = instr.get_icmp_predicate().ok_or("Missing ICmp predicate")?;
        let predicate = match predicate {
            inkwell::IntPredicate::EQ => BinaryOperator::Eq,
            inkwell::IntPredicate::NE => BinaryOperator::Ne,
            inkwell::IntPredicate::SLT => BinaryOperator::Lt,
            inkwell::IntPredicate::SGT => BinaryOperator::Gt,
            inkwell::IntPredicate::SLE => BinaryOperator::Le,
            inkwell::IntPredicate::SGE => BinaryOperator::Ge,
            _ => return Err("Unsupported ICmp predicate".to_string()),
        };

        self.gen_binary(instr, predicate)
    }

    fn gen_binary(
        &mut self,
        instr: &InstructionValue<'_>,
        op: BinaryOperator,
    ) -> Result<(), String> {
        if instr.get_num_operands() != 2 {
            return Err("Binary instruction must have 2 operands".to_string());
        }

        let dest_loc = self
            .allocator
            .get(&instr.as_value_ref())
            .ok_or("Destination location not found")?;
        let lhs_loc = self.allocator.get(
            &instr
                .get_operand(0)
                .ok_or("Binary instruction missing lhs operand")?
                .left()
                .ok_or("Invalid lhs operand")?
                .as_value_ref(),
        );
        let rhs_loc = self.allocator.get(
            &instr
                .get_operand(1)
                .ok_or("Binary instruction missing lhs operand")?
                .left()
                .ok_or("Invalid lhs operand")?
                .as_value_ref(),
        );

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
            BinaryOperator::Add => {
                self.builder.add(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Sub => {
                self.builder.sub(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Mul => {
                self.builder.mul(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Div => {
                self.builder.div(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::SRem => {
                self.builder.rem(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Lt => {
                self.builder.slt(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Gt => {
                self.builder.slt(RV32IReg::T0, RV32IReg::T1, RV32IReg::T0);
            }
            BinaryOperator::Le => {
                self.builder.slt(RV32IReg::T0, RV32IReg::T1, RV32IReg::T0);
                self.builder.seqz(RV32IReg::T0, RV32IReg::T0);
            }
            BinaryOperator::Ge => {
                self.builder.slt(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
                self.builder.seqz(RV32IReg::T0, RV32IReg::T0);
            }
            BinaryOperator::Eq => {
                self.builder.xor(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
                self.builder.seqz(RV32IReg::T0, RV32IReg::T0);
            }
            BinaryOperator::Ne => {
                self.builder.xor(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
                self.builder.snez(RV32IReg::T0, RV32IReg::T0);
            }
            BinaryOperator::And => {
                self.builder.and(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Or => {
                self.builder.or(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Xor => {
                self.builder.xor(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Ashr => {
                self.builder.sra(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Lshr => {
                self.builder.srl(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
            }
            BinaryOperator::Shl => {
                self.builder.sll(RV32IReg::T0, RV32IReg::T0, RV32IReg::T1);
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

    fn gen_return_sink(&mut self, is_entry: bool) -> Result<(), String> {
        if is_entry {
            self.builder.li(RV32IReg::A7, 93).ecall();
        } else {
            self.builder.ret();
        }
        Ok(())
    }

    fn gen_return(&mut self, instr: &InstructionValue<'_>, is_entry: bool) -> Result<(), String> {
        let num_operands = instr.get_num_operands();
        if num_operands == 0 {
            self.gen_return_sink(is_entry)?;
        }

        match self.allocator.get(
            &instr
                .get_operand(0)
                .ok_or("Return instruction missing operand")?
                .left()
                .ok_or("Invalid return operand")?
                .as_value_ref(),
        ) {
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
                let imm = instr
                    .get_operand(0)
                    .ok_or("Missing return operand")?
                    .left()
                    .ok_or("Invalid return operand")?
                    .into_int_value()
                    .get_zero_extended_constant()
                    .ok_or("Return value is not a constant")? as i32;
                self.builder.li(RV32IReg::A0, imm);
            }
        }
        self.gen_return_sink(is_entry)?;

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
