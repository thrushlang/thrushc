use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::llvm::compiler::{self, context::LLVMCodeGenContext};

use crate::frontends::classical::typesystem::types::Type;

use inkwell::{
    context::Context, targets::TargetData, types::BasicTypeEnum, values::BasicValueEnum,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alingof_type: &'ctx Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, alingof_type);

    let target_data: &TargetData = context.get_target_data();

    let memory_alignment: u32 = target_data.get_preferred_alignment(&llvm_type);

    let alignment: BasicValueEnum = llvm_context
        .i32_type()
        .const_int(memory_alignment.into(), false)
        .into();

    compiler::generation::cast::try_cast(context, cast, alingof_type, alignment)
        .unwrap_or(alignment)
}
