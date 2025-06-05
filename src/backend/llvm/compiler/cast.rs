use inkwell::{builder::Builder, context::Context, values::BasicValueEnum};

use crate::frontend::types::lexer::ThrushType;

use super::{context::LLVMCodeGenContext, typegen};

pub fn integer<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &ThrushType,
    from_type: &ThrushType,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::thrush_integer_to_llvm_type(llvm_context, target_type),
                true,
                "",
            )
            .unwrap()
            .into(),
    )
}

pub fn float<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &ThrushType,
    from_type: &ThrushType,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_float_cast(
                from.into_float_value(),
                typegen::type_float_to_llvm_float_type(llvm_context, target_type),
                "",
            )
            .unwrap()
            .into(),
    )
}
