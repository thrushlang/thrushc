use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::values::FunctionValue;

use thrushc_span::Span;

use crate::context::LLVMCodeGenContext;
use crate::obfuscation;
use crate::traits::LLVMFunctionExtensions;

#[inline]
pub fn move_terminator_to_end(context: &mut LLVMCodeGenContext, span: Span) {
    let function: FunctionValue = context.get_current_function(span).get_value();

    let last_builder_block: BasicBlock = context.get_last_builder_block(span);

    if let Some(parent) = function.get_last_basic_block() {
        let _ = last_builder_block.move_after(parent);
    }
}

#[inline]
pub fn move_specific_after_the_last(
    context: &mut LLVMCodeGenContext,
    block: BasicBlock,
    span: Span,
) {
    let last: BasicBlock = context.get_last_builder_block(span);
    let _ = block.move_after(last);
}

#[inline]
pub fn append_block<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    function: FunctionValue<'ctx>,
) -> BasicBlock<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let obfuscated_name: &str =
        &obfuscation::generate_string(context, obfuscation::SHORT_RANGE_OBFUSCATION);

    llvm_context.append_basic_block(function, obfuscated_name)
}
