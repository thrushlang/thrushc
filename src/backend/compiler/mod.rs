pub mod codegen;
pub mod functions;
pub mod general;
pub mod objects;
pub mod options;
pub mod utils;
pub mod variable;

use {
    super::instruction::Instruction,
    codegen::Codegen,
    inkwell::{builder::Builder, context::Context, module::Module},
    options::CompilerOptions,
};

pub struct Compiler;

impl<'a, 'ctx> Compiler {
    #[inline]
    pub fn compile(
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        context: &'ctx Context,
        options: &'a CompilerOptions,
        instructions: &'ctx [Instruction<'ctx>],
    ) {
        Codegen::gen(module, builder, context, options, instructions);
    }
}
