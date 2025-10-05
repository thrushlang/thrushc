use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::generation::cast;
use crate::backends::classical::llvm::compiler::{abort, typegen};

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum},
    values::BasicValueEnum,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    sizeof_type: &Type,
    cast: Option<&Type>,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, sizeof_type);

    let sizeof_value: BasicValueEnum = llvm_type
        .size_of()
        .unwrap_or_else(|| {
            abort::abort_codegen(
                context,
                "Failed to compile 'sizeof' builtin!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        })
        .into();

    cast::try_cast(context, cast, sizeof_type, sizeof_value, span).unwrap_or(sizeof_value)
}
