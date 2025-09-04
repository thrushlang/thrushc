use inkwell::{context::Context, module::Module};

use crate::backends::classical::llvm::compiler::optimizations::{inline, size};

#[derive(Debug)]
pub struct LLVMCompilerOptimizer<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
}

impl<'a, 'ctx> LLVMCompilerOptimizer<'a, 'ctx> {
    pub fn new(module: &'a Module<'ctx>, context: &'ctx Context) -> Self {
        Self { module, context }
    }

    pub fn optimize(&self) {
        self.module.get_functions().for_each(|llvm_function| {
            inline::apply(self.context, llvm_function);
            size::apply(self.context, llvm_function);
        });
    }
}
