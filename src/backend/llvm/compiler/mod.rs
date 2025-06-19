pub mod alloc;
pub mod array;
pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod cast;
pub mod codegen;
pub mod context;
pub mod conventions;
pub mod floatgen;
pub mod intgen;
pub mod intrinsics;
pub mod jit;
pub mod lli;
pub mod local;
pub mod memory;
pub mod mutation;
pub mod optimizations;
pub mod predicates;
pub mod rawgen;
pub mod typegen;
pub mod unaryop;
pub mod utils;
pub mod valuegen;

use {
    crate::{
        backend::llvm::compiler::context::LLVMCodeGenContext,
        frontend::types::parser::stmts::stmt::ThrushStatement,
    },
    codegen::LLVMCodegen,
};

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(
        context: &'a mut LLVMCodeGenContext<'a, 'ctx>,
        stmts: &'ctx [ThrushStatement<'ctx>],
    ) {
        LLVMCodegen::generate(context, stmts);
    }
}
