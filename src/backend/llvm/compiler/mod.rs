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
        standard::diagnostic::Diagnostician, types::frontend::parser::stmts::stmt::ThrushStatement,
    },
    codegen::LLVMCodegen,
    inkwell::{builder::Builder, context::Context, module::Module, targets::TargetData},
};

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        stmts: &'ctx [ThrushStatement<'ctx>],
        target_data: TargetData,
        diagnostician: Diagnostician,
    ) {
        LLVMCodegen::generate(module, builder, context, stmts, target_data, diagnostician);
    }
}
