pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod cast;
pub mod codegen;
pub mod context;
pub mod conventions;
pub mod llis;
pub mod local;
pub mod memory;
pub mod passes;
pub mod predicates;
pub mod typegen;
pub mod unaryop;
pub mod utils;
pub mod valuegen;

use {
    crate::{
        backend::llvm::compiler::context::LLVMCodeGenContext,
        types::frontend::parser::stmts::stmt::ThrushStatement,
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
