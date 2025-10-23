use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    llvm_sys::prelude::LLVMValueRef,
    module::Module,
    values::{
        AnyValue, AsValueRef, FunctionValue, GenericValue, GlobalValue, InstructionOpcode,
        InstructionValue,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RV32IReg {
    Zero,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
}

impl std::fmt::Display for RV32IReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal = match self {
            RV32IReg::Zero => "zero",
            RV32IReg::Ra => "ra",
            RV32IReg::Sp => "sp",
            RV32IReg::Gp => "gp",
            RV32IReg::Tp => "tp",
            RV32IReg::T0 => "t0",
            RV32IReg::T1 => "t1",
            RV32IReg::T2 => "t2",
            RV32IReg::S0 => "s0",
            RV32IReg::S1 => "s1",
            RV32IReg::A0 => "a0",
            RV32IReg::A1 => "a1",
            RV32IReg::A2 => "a2",
            RV32IReg::A3 => "a3",
            RV32IReg::A4 => "a4",
            RV32IReg::A5 => "a5",
            RV32IReg::A6 => "a6",
            RV32IReg::A7 => "a7",
            RV32IReg::S2 => "s2",
            RV32IReg::S3 => "s3",
            RV32IReg::S4 => "s4",
            RV32IReg::S5 => "s5",
            RV32IReg::S6 => "s6",
            RV32IReg::S7 => "s7",
            RV32IReg::S8 => "s8",
            RV32IReg::S9 => "s9",
            RV32IReg::S10 => "s10",
            RV32IReg::S11 => "s11",
            RV32IReg::T3 => "t3",
            RV32IReg::T4 => "t4",
            RV32IReg::T5 => "t5",
            RV32IReg::T6 => "t6",
        };
        write!(f, "{}", literal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    Stack(usize),
    Reg(RV32IReg),
    Global(String),
}

pub trait RegisterAllocator {
    fn new(module: &Module) -> Self;
    fn alloc(&mut self, func: &FunctionValue) -> Result<(), String>;
    fn stack_size_required(&self) -> usize;
    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location>;
}
pub struct NoneRegisterAllocator {
    global: HashMap<LLVMValueRef, Location>,
    vreg_map: HashMap<LLVMValueRef, Location>,
    stack_offset: usize,
}

impl<'ctx> RegisterAllocator for NoneRegisterAllocator {
    fn new(module: &Module) -> Self {
        let mut ret = Self {
            global: HashMap::new(),
            vreg_map: HashMap::new(),
            stack_offset: 0,
        };
        module.get_globals().for_each(|global| {
            ret.global.insert(
                global.as_value_ref(),
                Location::Global(global.get_name().to_str().unwrap_or_default().to_string()),
            );
        });
        ret
    }

    fn stack_size_required(&self) -> usize {
        // align to 16 bytes
        (self.stack_offset + 15) & !15
    }

    fn alloc(&mut self, func: &FunctionValue) -> Result<(), String> {
        self.vreg_map.clear();
        self.stack_offset = 0;
        func.get_basic_block_iter()
            .flat_map(|bb| bb.get_instructions())
            .for_each(|inst| {
                if !inst.get_type().is_void_type() {
                    self.vreg_map
                        .insert(inst.as_value_ref(), Location::Stack(self.stack_offset));
                    self.stack_offset += 4;
                }
            });

        Ok(())
    }

    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location> {
        self.vreg_map
            .get(val_ref)
            .or_else(|| self.global.get(val_ref))
            .cloned()
    }
}

pub struct LinearScanRegisterAllocator {
    global: HashMap<LLVMValueRef, Location>,
    vreg_map: HashMap<LLVMValueRef, Location>,
    stack_offset: usize,
}

impl<'ctx> RegisterAllocator for LinearScanRegisterAllocator {
    fn new(module: &Module) -> Self {
        let mut ret = Self {
            global: HashMap::new(),
            vreg_map: HashMap::new(),
            stack_offset: 0,
        };
        module.get_globals().for_each(|global| {
            ret.global.insert(
                global.as_value_ref(),
                Location::Global(global.get_name().to_str().unwrap_or_default().to_string()),
            );
        });
        ret
    }

    fn stack_size_required(&self) -> usize {
        // align to 16 bytes
        (self.stack_offset + 15) & !15
    }

    fn alloc(&mut self, func: &FunctionValue) -> Result<(), String> {
        self.vreg_map.clear();
        self.stack_offset = 0;
        func.get_basic_block_iter()
            .flat_map(|bb| bb.get_instructions())
            .for_each(|inst| {
                if !inst.get_type().is_void_type() {
                    self.vreg_map
                        .insert(inst.as_value_ref(), Location::Stack(self.stack_offset));
                    self.stack_offset += 4;
                }
            });

        Ok(())
    }

    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location> {
        self.vreg_map
            .get(val_ref)
            .or_else(|| self.global.get(val_ref))
            .cloned()
    }
}
