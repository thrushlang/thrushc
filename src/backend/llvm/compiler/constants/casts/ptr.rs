use std::fmt::Display;

use inkwell::{
    AddressSpace,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
};

pub fn const_ptr_cast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    cast: BasicTypeEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    if value.is_pointer_value() {
        let pointer: PointerValue = value.into_pointer_value();

        return pointer.const_cast(cast.into_pointer_type()).into();
    }

    self::codegen_abort("Cannot cast constant pointer value to non-basic type.");
    self::compile_null_ptr(context)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
