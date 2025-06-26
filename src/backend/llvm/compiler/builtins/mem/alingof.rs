use inkwell::{
    context::Context, targets::TargetData, types::BasicTypeEnum, values::BasicValueEnum,
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    frontend::types::lexer::ThrushType,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alingof_type: &'ctx ThrushType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, alingof_type);

    let target_data: &TargetData = context.get_target_data();

    let alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

    llvm_context
        .i32_type()
        .const_int(alignment.into(), false)
        .into()
}
