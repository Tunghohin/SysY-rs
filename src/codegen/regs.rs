use std::collections::HashMap;

use inkwell::{
    llvm_sys::prelude::LLVMValueRef,
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
    fn new() -> Self;
    fn alloc_global(&mut self, global_val: &GlobalValue) -> Result<(), String>;
    fn alloc_in_function(&mut self, name: &FunctionValue) -> Result<(), String>;
    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location>;
}

pub struct LinearScanRegisterAllocator {}

impl<'ctx> RegisterAllocator for LinearScanRegisterAllocator {
    fn new() -> Self {
        unimplemented!()
    }

    fn alloc_global(&mut self, name: &GlobalValue) -> Result<(), String> {
        unimplemented!()
    }

    fn alloc_in_function(&mut self, global_val: &FunctionValue) -> Result<(), String> {
        unimplemented!()
    }

    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location> {
        unimplemented!()
    }
}

pub struct NoneRegisterAllocator {
    global: HashMap<LLVMValueRef, Location>,
    vreg_map: HashMap<LLVMValueRef, Location>,
}

impl<'ctx> NoneRegisterAllocator {}

impl<'ctx> RegisterAllocator for NoneRegisterAllocator {
    fn new() -> Self {
        Self {
            global: HashMap::new(),
            vreg_map: HashMap::new(),
        }
    }

    fn alloc_global(&mut self, global_val: &GlobalValue) -> Result<(), String> {
        self.global.insert(
            global_val.as_value_ref(),
            Location::Global(global_val.get_name().to_str().unwrap().to_string()),
        );
        Ok(())
    }

    fn alloc_in_function(&mut self, name: &FunctionValue) -> Result<(), String> {
        let mut offset = 0usize;
        for bb in name.get_basic_blocks() {
            for inst in bb.get_instructions() {
                if !inst.get_type().is_void_type() {
                    self.vreg_map
                        .insert(inst.as_value_ref(), Location::Stack(offset));
                    offset += 4;
                }
            }
        }
        Ok(())
    }

    fn get(&self, val_ref: &LLVMValueRef) -> Option<Location> {
        self.vreg_map
            .get(val_ref)
            .or_else(|| self.global.get(val_ref))
            .cloned()
    }
}
