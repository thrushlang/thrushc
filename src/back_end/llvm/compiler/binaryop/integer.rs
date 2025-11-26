#![allow(unnecessary_transmutes)]
#![allow(clippy::incompatible_msrv)]

use crate::back_end::llvm::compiler;
use crate::back_end::llvm::compiler::abort;
use crate::back_end::llvm::compiler::codegen;
use crate::back_end::llvm::compiler::constgen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::generation::cast;
use crate::back_end::llvm::compiler::predicates;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::parser::repr::BinaryOperation;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use inkwell::values::IntValue;

fn int_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if lhs.is_int_value() && rhs.is_int_value() {
        let lhs: IntValue = lhs.into_int_value();
        let rhs: IntValue = rhs.into_int_value();

        let (lhs, rhs) = cast::integer_together(context, lhs, rhs, span);

        return match operator {
            TokenType::Plus => llvm_builder
                .build_int_nsw_add(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '+' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Minus => llvm_builder
                .build_int_nsw_sub(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '-' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Star => llvm_builder
                .build_int_nsw_mul(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '*' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '/' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '/' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::LShift => llvm_builder
                .build_left_shift(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '<<' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::RShift => llvm_builder
                .build_right_shift(lhs, rhs, signatures.0 || signatures.1, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '>>' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Arith if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_rem(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '%' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),

            TokenType::Arith => llvm_builder
                .build_int_unsigned_rem(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '%' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),

            TokenType::Xor => llvm_builder
                .build_xor(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '^' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::Bor => llvm_builder
                .build_or(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '|' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),
            TokenType::BAnd => llvm_builder
                .build_and(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile '&' operation!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),

            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(context, operator, signatures.0, signatures.1, span),
                    lhs,
                    rhs,
                    "",
                )
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile comparison!",
                        span,
                        PathBuf::from(file!()),
                        line!(),
                    );
                })
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return llvm_builder
                        .build_and(lhs, rhs, "")
                        .unwrap_or_else(|_| {
                            abort::abort_codegen(
                                context,
                                "Failed to compile '&&' operation!",
                                span,
                                PathBuf::from(file!()),
                                line!(),
                            );
                        })
                        .into();
                }

                if let TokenType::Or = op {
                    return llvm_builder
                        .build_or(lhs, rhs, "")
                        .unwrap_or_else(|_| {
                            abort::abort_codegen(
                                context,
                                "Failed to compile '||' operation!",
                                span,
                                PathBuf::from(file!()),
                                line!(),
                            );
                        })
                        .into();
                }

                abort::abort_codegen(
                    context,
                    "Failed to compile without a valid operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            }

            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant integer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::Arith
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or
        | TokenType::Xor
        | TokenType::Bor
        | TokenType::BAnd,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = codegen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = codegen::compile(context, binary.2, cast);

        let lhs_type: &Type = binary.0.llvm_get_type(context);
        let rhs_type: &Type = binary.2.llvm_get_type(context);

        return int_operation(
            context,
            lhs,
            rhs,
            (
                lhs_type.is_signed_integer_type(),
                rhs_type.is_signed_integer_type(),
            ),
            operator,
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile integer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

fn const_int_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
    span: Span,
) -> BasicValueEnum<'ctx> {
    if lhs.is_int_value() && rhs.is_int_value() {
        let lhs: IntValue = lhs.into_int_value();
        let rhs: IntValue = rhs.into_int_value();

        let (lhs, rhs) = compiler::generation::cast::const_integer_together(lhs, rhs, signatures);

        return match operator {
            TokenType::Plus => lhs.const_nsw_add(rhs).into(),
            TokenType::Minus => lhs.const_nsw_sub(rhs).into(),
            TokenType::Star => lhs.const_nsw_mul(rhs).into(),
            TokenType::Slash => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(
                                    unsafe {
                                        std::mem::transmute::<i64, u64>(
                                            lhs_number.overflowing_div(rhs_number).0,
                                        )
                                    },
                                    true,
                                )
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(lhs_number.overflowing_div(rhs_number).0, false)
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }
            TokenType::LShift => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(
                                    unsafe {
                                        std::mem::transmute::<i64, u64>(lhs_number.unbounded_shl(
                                            rhs_number.try_into().unwrap_or_default(),
                                        ))
                                    },
                                    true,
                                )
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(
                                lhs_number.unbounded_shl(rhs_number.try_into().unwrap_or_default()),
                                false,
                            )
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }
            TokenType::RShift => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(
                                    unsafe {
                                        std::mem::transmute::<i64, u64>(lhs_number.unbounded_shr(
                                            rhs_number.try_into().unwrap_or_default(),
                                        ))
                                    },
                                    true,
                                )
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(
                                lhs_number.unbounded_shr(rhs_number.try_into().unwrap_or_default()),
                                false,
                            )
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }
            TokenType::Arith => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int((lhs_number % rhs_number) as u64, true)
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(lhs_number % rhs_number, false)
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }
            TokenType::Xor => lhs.const_xor(rhs).into(),

            TokenType::Bor => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(
                                    unsafe {
                                        std::mem::transmute::<i64, u64>(lhs_number | rhs_number)
                                    },
                                    true,
                                )
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(lhs_number | rhs_number, false)
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }

            TokenType::BAnd => {
                if signatures.0 || signatures.1 {
                    if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                        if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(
                                    unsafe {
                                        std::mem::transmute::<i64, u64>(lhs_number & rhs_number)
                                    },
                                    true,
                                )
                                .into();
                        }
                    }
                }

                if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                    if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                        return lhs
                            .get_type()
                            .const_int(lhs_number & rhs_number, false)
                            .into();
                    }
                }

                lhs.get_type().const_zero().into()
            }

            op if op.is_logical_operator() => lhs
                .const_int_compare(
                    predicates::integer(context, operator, signatures.0, signatures.1, span),
                    rhs,
                )
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    if signatures.0 || signatures.1 {
                        if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                            if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                                return lhs
                                    .get_type()
                                    .const_int(
                                        ((lhs_number != 0) && (rhs_number != 0)) as u64,
                                        false,
                                    )
                                    .into();
                            }
                        }
                    }

                    if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                        if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(((lhs_number != 0) && (rhs_number != 0)) as u64, false)
                                .into();
                        }
                    }

                    return lhs.get_type().const_zero().into();
                }

                if let TokenType::Or = op {
                    if signatures.0 || signatures.1 {
                        if let Some(lhs_number) = lhs.get_sign_extended_constant() {
                            if let Some(rhs_number) = rhs.get_sign_extended_constant() {
                                return lhs
                                    .get_type()
                                    .const_int(
                                        ((lhs_number != 0) || (rhs_number != 0)) as u64,
                                        false,
                                    )
                                    .into();
                            }
                        }
                    }

                    if let Some(lhs_number) = lhs.get_zero_extended_constant() {
                        if let Some(rhs_number) = rhs.get_zero_extended_constant() {
                            return lhs
                                .get_type()
                                .const_int(((lhs_number != 0) || (rhs_number != 0)) as u64, false)
                                .into();
                        }
                    }

                    return lhs.get_type().const_zero().into();
                }

                abort::abort_codegen(
                    context,
                    "Failed to compile without a valid operator!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            }

            _ => abort::abort_codegen(
                context,
                "Failed to compile without a valid operator!",
                span,
                PathBuf::from(file!()),
                line!(),
            ),
        };
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant integer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::Plus
        | TokenType::Slash
        | TokenType::Minus
        | TokenType::Star
        | TokenType::Arith
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or
        | TokenType::Xor
        | TokenType::Bor
        | TokenType::BAnd,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        let lhs_type: &Type = binary.0.llvm_get_type(context);
        let rhs_type: &Type = binary.2.llvm_get_type(context);

        return self::const_int_operation(
            context,
            lhs,
            rhs,
            (
                rhs_type.is_signed_integer_type(),
                lhs_type.is_signed_integer_type(),
            ),
            operator,
            span,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant integer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}
