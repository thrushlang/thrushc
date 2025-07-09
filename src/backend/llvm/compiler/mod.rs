pub mod alloc;
pub mod anchors;
pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod cast;
pub mod codegen;
pub mod constants;
pub mod constgen;
pub mod context;
pub mod conventions;
pub mod expressions;
pub mod floatgen;
pub mod indexes;
pub mod intgen;
pub mod memory;
pub mod optimizations;
pub mod predicates;
pub mod ptrgen;
pub mod statements;
pub mod structgen;
pub mod typegen;
pub mod utils;
pub mod valuegen;

use {
    crate::{backend::llvm::compiler::context::LLVMCodeGenContext, frontend::types::ast::Ast},
    codegen::LLVMCodegen,
};

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMCodegen::generate(context, ast);
    }
}
