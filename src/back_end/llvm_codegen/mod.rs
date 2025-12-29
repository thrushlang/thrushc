pub mod abort;
pub mod allocate;
pub mod atomic;
pub mod attrbuilder;
pub mod attributes;
pub mod block;
pub mod builtins;
pub mod callconventions;
pub mod codegen;
pub mod codemodel;
pub mod constants;
pub mod constgen;
pub mod context;
pub mod debug;
pub mod declarations;
pub mod generation;
pub mod helpertypes;
pub mod indexes;
pub mod jit;
pub mod localanchor;
pub mod loopcontrol;
pub mod memory;
pub mod metadata;
pub mod obfuscation;
pub mod optimization;
pub mod predicates;
pub mod relocmodel;
pub mod statements;
pub mod symbolstable;
pub mod targettriple;
pub mod typegeneration;

use crate::back_end::llvm_codegen::codegen::LLVMCodegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::metadata::LLVMMetadata;

use crate::front_end::types::ast::Ast;

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMMetadata::setup(context);
        LLVMCodegen::generate(context, ast);
    }
}
