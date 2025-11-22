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
pub mod jit;
pub mod memory;
pub mod metadata;
pub mod obfuscation;
pub mod optimization;
pub mod predicates;
pub mod ptr;
pub mod statements;
pub mod symbols;
pub mod typegen;

use {
    crate::{
        back_end::llvm::compiler::{context::LLVMCodeGenContext, metadata::LLVMMetadata},
        front_end::types::ast::Ast,
    },
    codegen::LLVMCodegen,
};

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMMetadata::setup(context);
        LLVMCodegen::generate(context, ast);
    }
}
