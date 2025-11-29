use std::path::PathBuf;

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::{back_end::llvm::compiler::abort, core::diagnostic::span::Span};

use inkwell::{types::BasicTypeEnum, values::PointerValue};

#[inline]
pub fn try_alloc_heap<'ctx>(
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
        "Failed to allocate!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}
