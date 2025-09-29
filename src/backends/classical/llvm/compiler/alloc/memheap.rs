use std::path::PathBuf;

use crate::backends::classical::llvm::compiler::abort;
use crate::{
    backends::classical::llvm::compiler::context::LLVMCodeGenContext,
    frontends::classical::lexer::span::Span,
};

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
