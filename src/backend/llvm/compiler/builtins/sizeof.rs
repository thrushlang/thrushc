use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum},
    values::BasicValueEnum,
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    core::console::logging::{self, LoggingType},
    frontend::types::lexer::ThrushType,
};

pub fn compile<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    sizeof_type: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, sizeof_type);

    llvm_type
        .size_of()
        .unwrap_or_else(|| {
            logging::log(
                LoggingType::Bug,
                "Unable to get size of type at executation of the sizeof builtin.",
            );
            unreachable!()
        })
        .into()
}
