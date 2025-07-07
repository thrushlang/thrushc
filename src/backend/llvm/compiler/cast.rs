use std::cmp::Ordering;

use inkwell::{
    builder::Builder,
    context::Context,
    types::FloatType,
    values::{BasicValueEnum, FloatValue, IntValue},
};

use crate::frontend::typesystem::{traits::TypeMutableExtensions, types::Type};

use super::{context::LLVMCodeGenContext, typegen};

/* ######################################################################


    INTEGER CAST (TOGETHER)


########################################################################*/

pub fn const_integer_together<'ctx>(
    left: IntValue<'ctx>,
    right: IntValue<'ctx>,
    signatures: (bool, bool),
) -> (IntValue<'ctx>, IntValue<'ctx>) {
    let left_is_signed: bool = signatures.0;
    let right_is_signed: bool = signatures.1;

    match left
        .get_type()
        .get_bit_width()
        .cmp(&right.get_type().get_bit_width())
    {
        Ordering::Greater => {
            let new_right: IntValue<'ctx> = right.const_cast(left.get_type(), right_is_signed);

            (left, new_right)
        }
        Ordering::Less => {
            let new_left: IntValue<'ctx> = left.const_cast(right.get_type(), left_is_signed);

            (new_left, right)
        }

        _ => (left, right),
    }
}

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

/* ######################################################################


    FLOAT CAST (TOGETHER)


########################################################################*/

pub fn const_float_together<'ctx>(
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
) -> (FloatValue<'ctx>, FloatValue<'ctx>) {
    let left_type: FloatType = left.get_type();
    let right_type: FloatType = right.get_type();

    if left_type == right_type {
        return (left, right);
    }

    let new_left: FloatValue = if left_type != right_type {
        left.const_cast(right_type)
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        right.const_cast(left_type)
    } else {
        right
    };

    (new_left, new_right)
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

/* ######################################################################


    INTEGER CAST


########################################################################*/

pub fn integer<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let target_type: Type = target_type.defer_mut_all();

    if !from_type.is_integer_type() || !target_type.is_integer_type() {
        return None;
    }

    if *from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::integer_to_llvm_type(llvm_context, &target_type),
                from_type.is_signed_integer_type(),
                "",
            )
            .unwrap()
            .into(),
    )
}

/* ######################################################################


    FLOAT CAST


########################################################################*/

pub fn float<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let target_type: Type = target_type.defer_mut_all();

    if !from_type.is_float_type() || !target_type.is_float_type() {
        return None;
    }

    if *from_type == target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_float_cast(
                from.into_float_value(),
                typegen::type_float_to_llvm_float_type(llvm_context, &target_type),
                "",
            )
            .unwrap()
            .into(),
    )
}

/* ######################################################################


    INTELLIGENT CAST (TRY CAST)


########################################################################*/

pub fn try_cast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    if from.is_float_value() {
        return float(context, target_type, from_type, from);
    }

    if from.is_int_value() {
        return integer(context, target_type, from_type, from);
    }

    None
}
