pub mod attributes;
pub mod binaryop;
pub mod builtins;
pub mod call;
pub mod codegen;
pub mod conventions;
pub mod dealloc;
pub mod generation;
pub mod impls;
pub mod instruction;
pub mod local;
pub mod memory;
pub mod misc;
pub mod objects;
pub mod traits;
pub mod typegen;
pub mod types;
pub mod unaryop;
pub mod utils;
pub mod valuegen;

use {
    codegen::Codegen,
    inkwell::{builder::Builder, context::Context, module::Module, targets::TargetData},
    instruction::Instruction,
};

pub struct Compiler;

impl<'a, 'ctx> Compiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
        target_data: TargetData,
    ) {
        Codegen::generate(module, builder, context, instructions, target_data);
    }
}
