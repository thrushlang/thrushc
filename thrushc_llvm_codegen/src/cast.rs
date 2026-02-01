#![allow(unnecessary_transmutes)]

use inkwell::builder::Builder;
use inkwell::targets::TargetData;
use inkwell::types::BasicTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::PointerType;
use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;
use inkwell::values::IntValue;
use thrushc_ast::Ast;
use thrushc_ast::traits::AstCodeLocation;

use crate::abort;
use crate::codegen;
use crate::context::LLVMCodeGenContext;
use crate::traits::AstLLVMGetType;
use crate::typegeneration;

use thrushc_typesystem::traits::TypeIsExtensions;

use thrushc_span::Span;
use thrushc_typesystem::Type;

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
        std::cmp::Ordering::Greater => {
            if signatures.0 || signatures.1 {
                if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                    let lhs_number_transmuted: u64 =
                        unsafe { std::mem::transmute::<i64, u64>(lhs_number) };

                    return (rhs.get_type().const_int(lhs_number_transmuted, true), rhs);
                } else if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                    let rhs_number_transmuted: u64 =
                        unsafe { std::mem::transmute::<i64, u64>(rhs_number) };

                    return (lhs, lhs.get_type().const_int(rhs_number_transmuted, true));
                }
            }

            if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                return (rhs.get_type().const_int(lhs_number, true), rhs);
            }

            if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                return (lhs, lhs.get_type().const_int(rhs_number, true));
            }

            (rhs.get_type().const_zero(), rhs.get_type().const_zero())
        }
        std::cmp::Ordering::Less => {
            if signatures.0 || signatures.1 {
                if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                    let rhs_number_transmuted: u64 =
                        unsafe { std::mem::transmute::<i64, u64>(rhs_number) };

                    return (lhs, lhs.get_type().const_int(rhs_number_transmuted, true));
                } else if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                    let lhs_number_transmuted: u64 =
                        unsafe { std::mem::transmute::<i64, u64>(lhs_number) };

                    return (rhs.get_type().const_int(lhs_number_transmuted, true), rhs);
                }
            }

            if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                return (lhs, lhs.get_type().const_int(rhs_number, false));
            }

            if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                return (rhs.get_type().const_int(lhs_number, false), rhs);
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
        std::cmp::Ordering::Greater => {
            let new_right: IntValue = llvm_builder
                .build_int_cast_sign_flag(right, left.get_type(), false, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to cast integers together!",
                        span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                });

            (left, new_right)
        }
        std::cmp::Ordering::Less => {
            let new_left: IntValue = llvm_builder
                .build_int_cast_sign_flag(left, right.get_type(), false, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to cast integers together!",
                        span,
                        std::path::PathBuf::from(file!()),
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
        if let Some((n, ..)) = left.get_constant() {
            right.get_type().const_float(n)
        } else {
            left
        }
    } else {
        left
    };

    let new_right: FloatValue = if right_type != left_type {
        if let Some((n, ..)) = right.get_constant() {
            left.get_type().const_float(n)
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
                    std::path::PathBuf::from(file!()),
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
                    std::path::PathBuf::from(file!()),
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
                typegeneration::compile_from(context, target_type).into_int_type(),
                from_type.is_signed_integer_type(),
                "",
            )
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast integer!",
                    span,
                    std::path::PathBuf::from(file!()),
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
                typegeneration::compile_from(context, target_type).into_float_type(),
                "",
            )
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to cast float!",
                    span,
                    std::path::PathBuf::from(file!()),
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
) -> BasicValueEnum<'ctx> {
    if from.is_float_value()
        && let Some(target_type) = target_type
        && target_type.is_float_type()
    {
        return self::float(context, target_type, from_type, from, span).unwrap_or(from);
    } else if from.is_int_value()
        && let Some(target_type) = target_type
        && target_type.is_integer_type()
    {
        return self::integer(context, target_type, from_type, from, span).unwrap_or(from);
    }

    from
}

#[inline]
pub fn try_cast_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: BasicValueEnum<'ctx>,
    from_type: &Type,
    target_type: &Type,
) -> BasicValueEnum<'ctx> {
    match (from, target_type) {
        (lhs, rhs) if rhs.is_numeric_type() => {
            self::const_numeric_cast(context, lhs, rhs, from_type.is_signed_integer_type())
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
    expr: &'ctx Ast,
    target_type: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let from_type: &Type = expr.llvm_get_type();

    match (from_type, target_type) {
        (from, to) if from.is_ptr_like_type() && to.is_integer_type() => {
            let val: BasicValueEnum = codegen::compile_as_ptr(context, expr, None);

            if val.is_pointer_value() {
                let integer_type: BasicTypeEnum =
                    typegeneration::compile_from(context, target_type);

                return llvm_builder
                    .build_ptr_to_int(val.into_pointer_value(), integer_type.into_int_type(), "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into();
            };
        }

        (_, to) if to.is_numeric_type() => {
            let value: BasicValueEnum = codegen::compile(context, expr, None);
            let cast: BasicTypeEnum = typegeneration::compile_from(context, target_type);

            if self::is_same_bit_size(context, from_type, target_type) {
                return llvm_builder
                    .build_bit_cast(value, cast, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    });
            }

            if value.is_int_value() && cast.is_int_type() {
                return llvm_builder
                    .build_int_cast(value.into_int_value(), cast.into_int_type(), "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into();
            }

            if value.is_float_value() && target_type.is_float_type() {
                return llvm_builder
                    .build_float_cast(value.into_float_value(), cast.into_float_type(), "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into();
            }
        }

        (_, to) if to.is_ptr_type() => {
            let value: BasicValueEnum = codegen::compile_as_ptr(context, expr, None);

            if value.is_pointer_value() {
                let cast: PointerType =
                    typegeneration::compile_from(context, target_type).into_pointer_type();

                return llvm_builder
                    .build_pointer_cast(value.into_pointer_value(), cast, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into();
            };
        }

        (_, to) if to.is_const_type() => {
            let value: BasicValueEnum = codegen::compile_as_ptr(context, expr, None);

            if value.is_pointer_value() {
                let cast: PointerType =
                    typegeneration::compile_from(context, target_type).into_pointer_type();

                return llvm_builder
                    .build_pointer_cast(value.into_pointer_value(), cast, "")
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            context,
                            &format!(
                                "Failed to cast '{}' type to '{}' type.",
                                from_type, target_type
                            ),
                            expr.get_span(),
                            std::path::PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .into();
            };
        }

        _ => {}
    }

    abort::abort_codegen(
        context,
        &format!(
            "Failed to cast '{}' type to '{}' type.",
            from_type, target_type
        ),
        expr.get_span(),
        std::path::PathBuf::from(file!()),
        line!(),
    );
}

/* ######################################################################

    NUMERIC CAST (FLOAT & INT)

########################################################################*/

pub fn const_numeric_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    target: &Type,
    is_signed: bool,
) -> BasicValueEnum<'ctx> {
    let cast_type: BasicTypeEnum = typegeneration::compile_from(context, target);

    if value.is_int_value() && cast_type.is_int_type() {
        let integer: IntValue = value.into_int_value();
        let default_v: IntValue = cast_type.into_int_type().const_int(0, false);

        if is_signed {
            if let Some(n) = integer.get_sign_extended_constant() {
                let transmuted_n: u64 = unsafe { std::mem::transmute::<i64, u64>(n) };
                return cast_type
                    .into_int_type()
                    .const_int(transmuted_n, true)
                    .into();
            }
        }

        if let Some(number) = integer.get_zero_extended_constant() {
            return cast_type.into_int_type().const_int(number, false).into();
        }

        return default_v.into();
    }

    if value.is_float_value() && cast_type.is_float_type() {
        let float: FloatValue = value.into_float_value();

        return cast_type
            .into_float_type()
            .const_float(float.get_constant().unwrap_or((0.0, false)).0)
            .into();
    }

    value
}

#[inline]
fn is_same_bit_size(context: &mut LLVMCodeGenContext<'_, '_>, lhs: &Type, rhs: &Type) -> bool {
    let lhs: BasicTypeEnum = typegeneration::compile_from(context, lhs);
    let rhs: BasicTypeEnum = typegeneration::compile_from(context, rhs);

    let target_data: &TargetData = context.get_target_data();

    target_data.get_bit_size(&lhs) == target_data.get_bit_size(&rhs)
}
