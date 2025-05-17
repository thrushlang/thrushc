pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod codegen;
pub mod context;
pub mod conventions;
pub mod deallocator;
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
        middle::types::frontend::parser::stmts::instruction::Instruction,
        standard::diagnostic::Diagnostician,
    },
    codegen::LLVMCodegen,
    inkwell::{builder::Builder, context::Context, module::Module, targets::TargetData},
};

pub struct Compiler;

impl<'a, 'ctx> Compiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
        target_data: TargetData,
        diagnostician: Diagnostician,
    ) {
        LLVMCodegen::generate(
            module,
            builder,
            context,
            instructions,
            target_data,
            diagnostician,
        );
    }
}
