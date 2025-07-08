use std::fmt::Display;

use inkwell::{
    AddressSpace,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
};

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
};

pub fn ptr_cast<'ctx>(
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

pub fn numeric_cast<'ctx>(
    value: BasicValueEnum<'ctx>,
    cast: BasicTypeEnum<'ctx>,
    is_signed: bool,
) -> BasicValueEnum<'ctx> {
    if value.is_int_value() && cast.is_int_type() {
        let integer: IntValue = value.into_int_value();

        return integer.const_cast(cast.into_int_type(), is_signed).into();
    }

    if value.is_float_value() && cast.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return float.const_cast(cast.into_float_type()).into();
    }

    value
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
