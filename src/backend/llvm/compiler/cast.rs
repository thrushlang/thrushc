use std::cmp::Ordering;

use inkwell::{
    builder::Builder,
    context::Context,
    types::FloatType,
    values::{BasicValueEnum, FloatValue, IntValue},
};

use crate::frontend::types::lexer::ThrushType;

use super::{context::LLVMCodeGenContext, typegen};

pub fn integer_together<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: IntValue<'ctx>,
    right: IntValue<'ctx>,
) -> (IntValue<'ctx>, IntValue<'ctx>) {
    let llvm_builder: &Builder = context.get_llvm_builder();

    match left
        .get_type()
        .get_bit_width()
        .cmp(&right.get_type().get_bit_width())
    {
        Ordering::Greater => {
            let new_right: IntValue<'ctx> = llvm_builder
                .build_int_cast_sign_flag(right, left.get_type(), false, "")
                .unwrap();

            (left, new_right)
        }
        Ordering::Less => {
            let new_left: IntValue<'ctx> = llvm_builder
                .build_int_cast_sign_flag(left, right.get_type(), false, "")
                .unwrap();

            (new_left, right)
        }
        _ => (left, right),
    }
}

pub fn float_together<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
) -> (FloatValue<'ctx>, FloatValue<'ctx>) {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let left_type: FloatType = left.get_type();
    let right_type: FloatType = right.get_type();

    if left_type == right_type {
        return (left, right);
    }

    let new_left: FloatValue = if left_type != right_type {
        llvm_builder.build_float_cast(left, right_type, "").unwrap()
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        llvm_builder.build_float_cast(right, left_type, "").unwrap()
    } else {
        right
    };

    (new_left, new_right)
}
pub fn integer<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &ThrushType,
    from_type: &ThrushType,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if !(target_type.is_integer_type() || target_type.is_mut_integer_type())
        || !from_type.is_integer_type()
    {
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

    if !(target_type.is_float_type() || target_type.is_mut_float_type())
        || !from_type.is_float_type()
    {
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
