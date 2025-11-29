use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::{abort, typegen};

use crate::core::diagnostic::span::Span;

use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::{builder::Builder, context::Context, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc: &'ctx Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    llvm_builder
        .build_malloc(typegen::generate(llvm_context, alloc), "")
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                context,
                "Failed to compile 'halloc' builtin!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        })
        .into()
}
