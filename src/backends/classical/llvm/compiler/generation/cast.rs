use crate::backends::classical::llvm::compiler::{memory, ptr, typegen, value};
use crate::{
    backends::classical::llvm::compiler::context::LLVMCodeGenContext,
    frontends::classical::types::ast::Ast,
};

use crate::frontends::classical::typesystem::traits::{DereferenceExtensions, LLVMTypeExtensions};
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::{cmp::Ordering, fmt::Display};

use inkwell::types::{BasicTypeEnum, PointerType};
use inkwell::values::{PointerValue, StructValue};
use inkwell::{
    builder::Builder,
    context::Context,
    types::FloatType,
    values::{BasicValueEnum, FloatValue, IntValue},
};

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
    let abort = |_| {
        self::codegen_abort(format!("Failed to cast together '{}' & '{}'.", left, right));
    };

    let llvm_builder: &Builder = context.get_llvm_builder();

    match left
        .get_type()
        .get_bit_width()
        .cmp(&right.get_type().get_bit_width())
    {
        Ordering::Greater => {
            let new_right: IntValue<'ctx> = llvm_builder
                .build_int_cast_sign_flag(right, left.get_type(), false, "")
                .unwrap_or_else(abort);

            (left, new_right)
        }
        Ordering::Less => {
            let new_left: IntValue<'ctx> = llvm_builder
                .build_int_cast_sign_flag(left, right.get_type(), false, "")
                .unwrap_or_else(abort);

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
    let abort = |_| {
        self::codegen_abort(format!("Failed to cast together '{}' & '{}'.", left, right));
    };

    let llvm_builder: &Builder = context.get_llvm_builder();

    let left_type: FloatType = left.get_type();
    let right_type: FloatType = right.get_type();

    if left_type == right_type {
        return (left, right);
    }

    let new_left: FloatValue = if left_type != right_type {
        llvm_builder
            .build_float_cast(left, right_type, "")
            .unwrap_or_else(abort)
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        llvm_builder
            .build_float_cast(right, left_type, "")
            .unwrap_or_else(abort)
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
    let abort = |_| {
        self::codegen_abort(format!(
            "Failed to cast '{}' to '{}'.",
            from_type, target_type
        ));
    };

    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let target_type: Type = target_type.dereference_high_level_type();

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
            .unwrap_or_else(abort)
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
    let abort = |_| {
        self::codegen_abort(format!(
            "Failed to cast '{}' to '{}'.",
            from_type, target_type
        ));
    };

    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let target_type: Type = target_type.dereference_high_level_type();

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
                typegen::float_to_llvm_type(llvm_context, &target_type),
                "",
            )
            .unwrap_or_else(abort)
            .into(),
    )
}

/* ######################################################################


    INTELLIGENT CAST (TRY CAST)


########################################################################*/

#[inline]
pub fn try_cast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    target_type: Option<&Type>,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    if from.is_float_value() && target_type.is_some() {
        if let Some(target_type) = target_type {
            return self::float(context, target_type, from_type, from);
        }
    }

    if from.is_int_value() && target_type.is_some() {
        if let Some(target_type) = target_type {
            return self::integer(context, target_type, from_type, from);
        }
    }

    None
}

#[inline]
pub fn try_cast_const<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,

    lhs: BasicValueEnum<'ctx>,
    lhs_type: &Type,
    rhs: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match (lhs, rhs) {
        (lhs, rhs) if rhs.is_numeric() => {
            if lhs_type.llvm_is_same_bit_size(context, rhs) {
                self::const_numeric_bitcast_cast(context, lhs, rhs)
            } else {
                let cast: BasicTypeEnum = typegen::generate_type(llvm_context, rhs);
                self::numeric_cast(lhs, cast, lhs_type.is_signed_integer_type())
            }
        }

        (lhs, rhs) if rhs.is_ptr_type() => {
            let cast: BasicTypeEnum = typegen::generate_type(llvm_context, rhs);
            self::const_ptr_cast(lhs, cast)
        }

        _ => lhs,
    }
}

/* ######################################################################

    UNIVERSAL CAST (COMPILE CAST)

########################################################################*/

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,

    lhs: &'ctx Ast,
    rhs: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let lhs_type: &Type = lhs.get_type_unwrapped();

    let abort_ptr = |_| self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs));

    if lhs_type.is_str_type() && rhs.is_ptr_type() {
        let val: BasicValueEnum = ptr::compile(context, lhs, None);

        let raw_str_ptr: PointerValue = val.into_pointer_value();
        let str_loaded: BasicValueEnum = memory::load_anon(context, raw_str_ptr, lhs_type);
        let str_structure: StructValue = str_loaded.into_struct_value();

        match llvm_builder.build_extract_value(str_structure, 0, "") {
            Ok(cstr) => {
                let to = typegen::generate_type(llvm_context, rhs).into_pointer_type();

                match llvm_builder.build_pointer_cast(cstr.into_pointer_value(), to, "") {
                    Ok(casted_ptr) => return casted_ptr.into(),
                    Err(_) => {
                        self::codegen_abort(format!("Failed to cast string pointer in '{}'.", lhs))
                    }
                }
            }

            Err(_) => self::codegen_abort(format!("Failed to extract string value in '{}'.", lhs)),
        }
    }

    if rhs.is_numeric() {
        let value: BasicValueEnum = value::compile(context, lhs, None);
        let target_type: BasicTypeEnum = typegen::generate_type(llvm_context, rhs);

        if lhs_type.llvm_is_same_bit_size(context, rhs) {
            return llvm_builder
                .build_bit_cast(value, target_type, "")
                .unwrap_or_else(|_| {
                    self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs))
                });
        }

        if value.is_int_value() && target_type.is_int_type() {
            return llvm_builder
                .build_int_cast(value.into_int_value(), target_type.into_int_type(), "")
                .unwrap_or_else(|_| {
                    self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs))
                })
                .into();
        }

        if value.is_float_value() && target_type.is_float_type() {
            return llvm_builder
                .build_float_cast(value.into_float_value(), target_type.into_float_type(), "")
                .unwrap_or_else(|_| {
                    self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs))
                })
                .into();
        }
    }

    if rhs.is_ptr_type() {
        let value: BasicValueEnum = ptr::compile(context, lhs, None);

        if value.is_pointer_value() {
            let to: PointerType = typegen::generate_type(llvm_context, rhs).into_pointer_type();

            return llvm_builder
                .build_pointer_cast(value.into_pointer_value(), to, "")
                .unwrap_or_else(abort_ptr)
                .into();
        };
    }

    self::codegen_abort(format!(
        "Unsupported cast from '{}' to '{}'.",
        lhs_type, lhs_type
    ));
}

/* ######################################################################

    NUMERIC BITCAST CAST

########################################################################*/

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

/* ######################################################################

    POINTER BITCAST CAST

########################################################################*/

pub fn const_ptr_cast<'ctx>(
    value: BasicValueEnum<'ctx>,
    cast: BasicTypeEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    if value.is_pointer_value() {
        let pointer: PointerValue = value.into_pointer_value();

        return pointer.const_cast(cast.into_pointer_type()).into();
    }

    self::codegen_abort("Cannot cast constant pointer value to non-basic type.");
}

/* ######################################################################

    NUMERIC CAST (FLOAT & INT)

########################################################################*/

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

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
