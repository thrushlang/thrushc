pub mod binaryop;
pub mod call;
pub mod codegen;
pub mod generation;
pub mod misc;
pub mod objects;
pub mod types;
pub mod unaryop;
pub mod utils;
pub mod variable;

use {
    super::instruction::Instruction,
    codegen::Codegen,
    inkwell::{builder::Builder, context::Context, module::Module},
};

pub struct Compiler;

impl<'a, 'ctx> Compiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        instructions: &'ctx [Instruction<'ctx>],
    ) {
        Codegen::gen(module, builder, context, instructions);
    }
}
