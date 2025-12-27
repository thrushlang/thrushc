use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::obfuscation;
use crate::back_end::llvm_codegen::types::traits::LLVMFunctionExtensions;
use crate::core::diagnostic::span::Span;

use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::values::FunctionValue;

#[inline]
pub fn move_terminator_to_end(context: &mut LLVMCodeGenContext, span: Span) {
    let function: FunctionValue = context.get_current_llvm_function(span).get_value();

    let last_builder_block: BasicBlock = context.get_last_builder_block(span);

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
