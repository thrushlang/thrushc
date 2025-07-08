pub mod alloc;
pub mod anchors;
pub mod array;
pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod cast;
pub mod codegen;
pub mod conditional;
pub mod constants;
pub mod constgen;
pub mod context;
pub mod conventions;
pub mod farray;
pub mod floatgen;
pub mod indexes;
pub mod intgen;
pub mod intrinsics;
pub mod lli;
pub mod local;
pub mod loops;
pub mod memory;
pub mod mutation;
pub mod optimizations;
pub mod predicates;
pub mod ptrgen;
pub mod string;
pub mod structgen;
pub mod terminator;
pub mod typegen;
pub mod unaryop;
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
