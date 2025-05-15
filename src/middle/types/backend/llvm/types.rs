use ahash::AHashMap as HashMap;
use inkwell::values::{BasicValueEnum, FunctionValue};

use crate::{
    backend::llvm::compiler::memory::SymbolAllocated,
    middle::types::frontend::{
        lexer::{tokenkind::TokenKind, types::ThrushType},
        parser::stmts::{instruction::Instruction, types::ThrushAttributes},
    },
};

pub type SymbolsAllocated<'ctx> = &'ctx HashMap<&'ctx str, SymbolAllocated<'ctx>>;

pub type LLVMBinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type LLVMUnaryOp<'ctx> = (&'ctx TokenKind, &'ctx ThrushType, &'ctx Instruction<'ctx>);

pub type LLVMCall<'ctx> = (&'ctx ThrushType, BasicValueEnum<'ctx>);
pub type LLVMCalls<'ctx> = Vec<LLVMCall<'ctx>>;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [ThrushType], u32);

pub type LLVMFunctionPrototype<'ctx> = (
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [Instruction<'ctx>],
    &'ctx [ThrushType],
    &'ctx Instruction<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type LLVMFunctionCall<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx [Instruction<'ctx>]);

pub type LLVMFunctionParameter<'ctx> = (&'ctx str, &'ctx ThrushType, u32, bool);

pub type LLVMLocal<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx Instruction<'ctx>);
