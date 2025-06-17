use inkwell::{attributes::AttributeLoc, context::Context, module::Linkage, values::FunctionValue};

use crate::backend::llvm::compiler::attributes;

pub fn apply(llvm_context: &Context, llvm_function: FunctionValue) {
    let mut instructions_count: usize = 0;

    llvm_function.get_basic_block_iter().for_each(|block| {
        instructions_count += block.get_instructions().count();
    });

    if let Linkage::LinkerPrivate | Linkage::Internal | Linkage::Private =
        llvm_function.get_linkage()
    {
        if instructions_count <= 5 {
            llvm_function.add_attribute(
                AttributeLoc::Function,
                attributes::create_always_inline_attribute(llvm_context),
            );
        } else if instructions_count <= 30 {
            llvm_function.add_attribute(
                AttributeLoc::Function,
                attributes::create_inline_hint_attribute(llvm_context),
            );
        } else if instructions_count >= 80 {
            llvm_function.add_attribute(
                AttributeLoc::Function,
                attributes::create_minsize_attribute(llvm_context),
            );
        }
    }
}
