use inkwell::{
    types::FunctionType,
    values::{FunctionValue, PointerValue},
};

use crate::frontend::types::lexer::ThrushType;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [ThrushType], u32);

pub type LLVMAssemblerFunction<'ctx> = (FunctionType<'ctx>, PointerValue<'ctx>, &'ctx [ThrushType]);
