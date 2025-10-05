pub mod abort;
pub mod alloc;
pub mod anchors;
pub mod attrbuilder;
pub mod attributes;
pub mod binaryop;
pub mod block;
pub mod builtins;
pub mod codegen;
pub mod constants;
pub mod constgen;
pub mod context;
pub mod control;
pub mod conventions;
pub mod declarations;
pub mod generation;
pub mod indexes;
pub mod memory;
pub mod obfuscation;
pub mod optimization;
pub mod predicates;
pub mod ptr;
pub mod statements;
pub mod symbols;
pub mod typegen;

use {
    crate::{
        backends::classical::llvm::compiler::context::LLVMCodeGenContext,
        frontends::classical::types::ast::Ast,
    },
    codegen::LLVMCodegen,
};

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMCodegen::generate(context, ast);
    }
}
