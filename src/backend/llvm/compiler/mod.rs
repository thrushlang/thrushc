pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod codegen;
pub mod context;
pub mod conventions;
pub mod dealloc;
pub mod local;
pub mod memory;
pub mod predicates;
pub mod typegen;
pub mod types;
pub mod unaryop;
pub mod utils;
pub mod valuegen;

use {
    crate::{middle::instruction::Instruction, standard::diagnostic::Diagnostician},
    codegen::Codegen,
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
        Codegen::generate(
            module,
            builder,
            context,
            instructions,
            target_data,
            diagnostician,
        );
    }
}
