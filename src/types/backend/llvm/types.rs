use ahash::AHashMap as HashMap;
use inkwell::values::{BasicValueEnum, FunctionValue};

use crate::{
    backend::llvm::compiler::memory::SymbolAllocated,
    types::frontend::{
        lexer::{tokenkind::TokenKind, types::ThrushType},
        parser::stmts::{stmt::ThrushStatement, types::ThrushAttributes},
    },
};

pub type SymbolsAllocated<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;

pub type LLVMBinaryOp<'ctx> = (
    &'ctx ThrushStatement<'ctx>,
    &'ctx TokenKind,
    &'ctx ThrushStatement<'ctx>,
);

pub type LLVMUnaryOp<'ctx> = (
    &'ctx TokenKind,
    &'ctx ThrushType,
    &'ctx ThrushStatement<'ctx>,
);

pub type LLVMScopeCall<'ctx> = (&'ctx ThrushType, BasicValueEnum<'ctx>);
pub type LLVMScopeCalls<'ctx> = Vec<LLVMScopeCall<'ctx>>;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [ThrushType], u32);

pub type LLVMFunctionPrototype<'ctx> = (
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [ThrushStatement<'ctx>],
    &'ctx [ThrushType],
    &'ctx ThrushStatement<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type LLVMFunctionCall<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx [ThrushStatement<'ctx>]);

pub type LLVMFunctionParameter<'ctx> = (&'ctx str, &'ctx ThrushType, u32, bool);

pub type LLVMLocal<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx ThrushStatement<'ctx>);
