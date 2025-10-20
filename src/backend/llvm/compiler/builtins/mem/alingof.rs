use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::generation::cast;
use crate::backend::llvm::compiler::typegen;

use crate::frontend::lexer::span::Span;
use crate::frontend::typesystem::types::Type;

use inkwell::{
    context::Context, targets::TargetData, types::BasicTypeEnum, values::BasicValueEnum,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alingof_type: &'ctx Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, alingof_type);

    let target_data: &TargetData = context.get_target_data();

    let alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

    let alignment: BasicValueEnum = llvm_context
        .i32_type()
        .const_int(alignment.into(), false)
        .into();

    cast::try_cast(context, cast_type, alingof_type, alignment, span).unwrap_or(alignment)
}
