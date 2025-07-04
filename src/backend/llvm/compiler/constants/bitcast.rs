use inkwell::{
    context::Context,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FloatValue, IntValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    frontend::types::lexer::Type,
};

pub fn const_numeric_bitcast_cast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, cast);

    if value.is_int_value() && cast.is_integer_type() {
        let integer: IntValue = value.into_int_value();

        return integer.const_bit_cast(llvm_type.into_int_type()).into();
    }

    if value.is_float_value() && cast.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return float.const_cast(llvm_type.into_float_type()).into();
    }

    value
}
