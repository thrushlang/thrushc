use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::core::diagnostic::span::Span;

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use std::path::PathBuf;

#[inline]
pub fn try_alloc_at_heap<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    span: Span,
) -> PointerValue<'ctx> {
    if let Ok(ptr) = context
        .get_llvm_builder()
        .build_malloc(llvm_type, ascii_name)
    {
        return ptr;
    }

    abort::abort_codegen(
        context,
        "Failed to allocate at heap!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}
