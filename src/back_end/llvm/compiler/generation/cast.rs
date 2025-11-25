#![allow(unnecessary_transmutes)]

use crate::back_end::llvm::compiler::{abort, codegen, ptr, typegen};
use crate::front_end::lexer::span::Span;
use crate::{back_end::llvm::compiler::context::LLVMCodeGenContext, front_end::types::ast::Ast};

use crate::front_end::typesystem::traits::LLVMTypeExtensions;
use crate::front_end::typesystem::types::Type;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::path::PathBuf;
use std::{cmp::Ordering, fmt::Display};

use inkwell::types::{BasicTypeEnum, PointerType};
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
    lhs: IntValue<'ctx>,
    rhs: IntValue<'ctx>,
    signatures: (bool, bool),
) -> (IntValue<'ctx>, IntValue<'ctx>) {
    match lhs
        .get_type()
        .get_bit_width()
        .cmp(&rhs.get_type().get_bit_width())
    {
        Ordering::Greater => {
            if signatures.0 || signatures.1 {
                if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                    return (
                        rhs.get_type().const_int(
                            unsafe { std::mem::transmute::<i64, u64>(lhs_number) },
                            true,
                        ),
                        rhs,
                    );
                }
            }

            if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                return (rhs.get_type().const_int(lhs_number, true), rhs);
            }

            (rhs.get_type().const_zero(), rhs.get_type().const_zero())
        }
        Ordering::Less => {
            if signatures.0 || signatures.1 {
                if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                    return (
                        lhs,
                        lhs.get_type().const_int(
                            unsafe { std::mem::transmute::<i64, u64>(rhs_number) },
                            true,
                        ),
                    );
                }
            }

            if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                return (lhs, lhs.get_type().const_int(rhs_number, true));
            }

            (rhs.get_type().const_zero(), rhs.get_type().const_zero())
        }

        _ => (lhs, rhs),
    }
}

pub fn integer_together<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: IntValue<'ctx>,
    right: IntValue<'ctx>,
    span: Span,
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
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to cast integers together!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    )
                });

            (left, new_right)
        }
        Ordering::Less => {
            let new_left: IntValue<'ctx> = llvm_builder
                .build_int_cast_sign_flag(left, right.get_type(), false, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to cast integers together!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    )
                });

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
        if let Some((left_number, ..)) = left.get_constant() {
            right.get_type().const_float(left_number)
        } else {
            left
        }
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        if let Some((right_number, ..)) = right.get_constant() {
            left.get_type().const_float(right_number)
        } else {
            right
        }
    } else {
        right
    };

    (new_left, new_right)
}

pub fn float_together<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
    span: Span,
) -> (FloatValue<'ctx>, FloatValue<'ctx>) {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let left_type: FloatType = left.get_type();
    let right_type: FloatType = right.get_type();

    if left_type == right_type {
        return (left, right);
    }

    let new_left: FloatValue = if left_type != right_type {
        llvm_builder
            .build_float_cast(left, right_type, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast floats together!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        llvm_builder
            .build_float_cast(right, left_type, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast floats together!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
    } else {
        right
    };

    (new_left, new_right)
}

/* ######################################################################


    INTEGER CAST


########################################################################*/

pub fn integer<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
    span: Span,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if !from_type.is_integer_type() || !target_type.is_integer_type() {
        return None;
    }

    if *from_type == *target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::generate(llvm_context, target_type).into_int_type(),
                from_type.is_signed_integer_type(),
                "",
            )
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast integer!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
    )
}

/* ######################################################################


    FLOAT CAST


########################################################################*/

pub fn float<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
    span: Span,
) -> Option<BasicValueEnum<'ctx>> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if !from_type.is_float_type() || !target_type.is_float_type() {
        return None;
    }

    if *from_type == *target_type {
        return None;
    }

    Some(
        llvm_builder
            .build_float_cast(
                from.into_float_value(),
                typegen::generate(llvm_context, target_type).into_float_type(),
                "",
            )
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast float!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
    )
}

/* ######################################################################


    INTELLIGENT CAST (TRY CAST)


########################################################################*/

#[inline]
pub fn try_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    target_type: Option<&Type>,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
    span: Span,
) -> Option<BasicValueEnum<'ctx>> {
    if from.is_float_value() && target_type.is_some() {
        if let Some(target_type) = target_type {
            return self::float(context, target_type, from_type, from, span);
        }
    }

    if from.is_int_value() && target_type.is_some() {
        if let Some(target_type) = target_type {
            return self::integer(context, target_type, from_type, from, span);
        }
    }

    None
}

#[inline]
pub fn try_cast_const<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    from: BasicValueEnum<'ctx>,
    from_type: &Type,
    target_type: &Type,
) -> BasicValueEnum<'ctx> {
    match (from, target_type) {
        (lhs, rhs) if rhs.is_numeric_type() => {
            if from_type.llvm_is_same_bit_size(context, rhs) {
                self::const_numeric_bitcast_cast(context, lhs, rhs)
            } else {
                self::numeric_cast(context, lhs, rhs, from_type.is_signed_integer_type())
            }
        }

        (lhs, rhs) if lhs.is_pointer_value() && rhs.is_ptr_type() => {
            lhs.into_pointer_value().into()
        }

        _ => from,
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

    let lhs_type: &Type = lhs.llvm_get_type(context);

    let abort_ptrtoint =
        |_| self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs));

    let abort_ptr = |_| self::codegen_abort(format!("Failed to cast '{}' to '{}'.", lhs_type, rhs));

    if lhs_type.is_ptr_type() && rhs.is_integer_type() {
        let val: BasicValueEnum = ptr::compile(context, lhs, None);

        if val.is_pointer_value() {
            let integer_type: BasicTypeEnum = typegen::generate(llvm_context, rhs);

            return llvm_builder
                .build_ptr_to_int(val.into_pointer_value(), integer_type.into_int_type(), "")
                .unwrap_or_else(abort_ptrtoint)
                .into();
        };

        self::codegen_abort(format!(
            "Failed to cast pointer '{}' to integer '{}'.",
            lhs, rhs
        ));
    }

    if rhs.is_numeric_type() {
        let value: BasicValueEnum = codegen::compile(context, lhs, None);
        let target_type: BasicTypeEnum = typegen::generate(llvm_context, rhs);

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
            let to: PointerType = typegen::generate(llvm_context, rhs).into_pointer_type();

            return llvm_builder
                .build_pointer_cast(value.into_pointer_value(), to, "")
                .unwrap_or_else(abort_ptr)
                .into();
        };
    }

    if lhs_type.is_ptr_type() && rhs.is_const_type() {
        let value: BasicValueEnum = ptr::compile(context, lhs, None);

        if value.is_pointer_value() {
            let to: PointerType = typegen::generate(llvm_context, rhs).into_pointer_type();

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

    let cast: BasicTypeEnum = typegen::generate(llvm_context, cast);

    if value.is_int_value() && cast.is_int_type() {
        let integer: IntValue = value.into_int_value();

        if let Some(number) = integer.get_sign_extended_constant() {
            return cast
                .into_int_type()
                .const_int(unsafe { std::mem::transmute::<i64, u64>(number) }, true)
                .into();
        }

        if let Some(number) = integer.get_zero_extended_constant() {
            return cast.into_int_type().const_int(number, false).into();
        }

        return cast.into_int_type().const_int(0, false).into();
    }

    if value.is_float_value() && cast.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return cast
            .into_float_type()
            .const_float(float.get_constant().unwrap_or((0.0, false)).0)
            .into();
    }

    value
}

/* ######################################################################

    NUMERIC CAST (FLOAT & INT)

########################################################################*/

pub fn numeric_cast<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    target: &Type,
    is_signed: bool,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let cast: BasicTypeEnum = typegen::generate(llvm_context, target);

    if value.is_int_value() && cast.is_int_type() {
        let integer: IntValue = value.into_int_value();

        if is_signed {
            if let Some(number) = integer.get_sign_extended_constant() {
                return cast
                    .into_int_type()
                    .const_int(unsafe { std::mem::transmute::<i64, u64>(number) }, true)
                    .into();
            }
        }

        if let Some(number) = integer.get_zero_extended_constant() {
            return cast.into_int_type().const_int(number, false).into();
        }

        return cast.into_int_type().const_int(0, false).into();
    }

    if value.is_float_value() && cast.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return cast
            .into_float_type()
            .const_float(float.get_constant().unwrap_or((0.0, false)).0)
            .into();
    }

    value
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
