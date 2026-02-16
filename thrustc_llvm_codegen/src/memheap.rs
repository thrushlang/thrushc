use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use thrustc_span::Span;

use std::path::PathBuf;

use crate::abort;
use crate::context::LLVMCodeGenContext;

#[inline]
pub fn try_alloc_at_heap<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    span: Span,
) -> PointerValue<'ctx> {
    context.mark_dbg_location(span);

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
