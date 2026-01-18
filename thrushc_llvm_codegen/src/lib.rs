use thrushc_ast::Ast;

use crate::{codegen::LLVMCodegen, context::LLVMCodeGenContext, metadata::LLVMMetadata};

mod abort;
mod anchor;
mod atomic;
mod attrbuilder;
mod block;
mod brancher;
mod builtins;
mod cast;
mod codegen;
pub mod context;
mod debug;
mod expressions;
mod globals;
mod impls;
pub mod jit;
mod memheap;
mod memory;
mod memstack;
mod memstatic;
mod metadata;
mod obfuscation;
pub mod optimizer;
mod predicates;
mod statements;
mod table;
mod targettriple;
mod traits;
mod typegeneration;
mod types;
mod utils;

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMMetadata::setup_platform_independent(context);
        LLVMCodegen::generate(context, ast);
    }
}
