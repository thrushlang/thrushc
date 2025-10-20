use inkwell::{basic_block::BasicBlock, context::Context, values::FunctionValue};

use crate::backend::llvm::compiler::{context::LLVMCodeGenContext, obfuscation};

#[inline]
pub fn move_terminator_to_end(context: &LLVMCodeGenContext) {
    let function: FunctionValue = context.get_current_fn();

    let last_builder_block: BasicBlock = context.get_last_builder_block();

    if let Some(parent) = function.get_last_basic_block() {
        let _ = last_builder_block.move_after(parent);
    }
}

#[inline]
pub fn append_block<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    function: FunctionValue<'ctx>,
) -> BasicBlock<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let obfuscated_name: &str =
        &obfuscation::generate_obfuscation_name(context, obfuscation::SHORT_RANGE_OBFUSCATION);

    llvm_context.append_basic_block(function, obfuscated_name)
}
