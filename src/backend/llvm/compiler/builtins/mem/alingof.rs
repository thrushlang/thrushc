use inkwell::{
    context::Context, targets::TargetData, types::BasicTypeEnum, values::BasicValueEnum,
};

use crate::{
    backend::llvm::compiler::{cast, context::LLVMCodeGenContext, typegen},
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alingof_type: &'ctx Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, alingof_type);

    let target_data: &TargetData = context.get_target_data();

    let memory_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

    let alignment: BasicValueEnum = llvm_context
        .i32_type()
        .const_int(memory_alignment.into(), false)
        .into();

    cast::try_cast(context, cast, alingof_type, alignment).unwrap_or(alignment)
}
