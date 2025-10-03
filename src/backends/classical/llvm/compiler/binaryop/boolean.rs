use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::abort;
use crate::backends::classical::llvm::compiler::codegen;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::context::Context;
use inkwell::values::PointerValue;
use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, FloatValue, IntValue},
};

use std::fmt::Display;
use std::path::PathBuf;

pub fn bool_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let lhs_signed: bool = signatures.0;
    let rhs_signed: bool = signatures.1;

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform boolean binary operation.");
    };

    if lhs.is_int_value() && rhs.is_int_value() {
        let lhs: IntValue = lhs.into_int_value();
        let rhs: IntValue = rhs.into_int_value();

        let (lhs, rhs) = compiler::generation::cast::integer_together(context, lhs, rhs);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, lhs_signed, rhs_signed),
                    lhs,
                    rhs,
                    "",
                )
                .unwrap_or_else(cintgen_abort)
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return llvm_builder
                        .build_and(lhs, rhs, "")
                        .unwrap_or_else(cintgen_abort)
                        .into();
                }

                if let TokenType::Or = op {
                    return llvm_builder
                        .build_or(lhs, rhs, "")
                        .unwrap_or_else(cintgen_abort)
                        .into();
                }

                self::codegen_abort(
                    "Cannot perform boolean binary operation without a valid gate.",
                );
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform boolean binary operation without a valid operator.",
                );
            }
        };
    }

    if lhs.is_float_value() && rhs.is_float_value() {
        let lhs: FloatValue = lhs.into_float_value();
        let rhs: FloatValue = rhs.into_float_value();

        let (lhs, rhs) = compiler::generation::cast::float_together(context, lhs, rhs);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_float_compare(predicates::float(operator), lhs, rhs, "")
                .unwrap_or_else(cintgen_abort)
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform boolean binary operation without two float values.",
                );
            }
        };
    }

    if lhs.is_pointer_value() && rhs.is_pointer_value() {
        let lhs: PointerValue = lhs.into_pointer_value();
        let rhs: PointerValue = rhs.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(predicates::pointer(operator), lhs, rhs, "")
                .unwrap_or_else(|_| {
                    self::codegen_abort("Cannot perform pointer binary operation.");
                })
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform pointer binary operation without a valid operator.",
                );
            }
        };
    }

    self::codegen_abort("Cannot perform boolean binary operation without two integer values.");
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (
        _,
        TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::And
        | TokenType::Or,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = codegen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = codegen::compile(context, binary.2, cast);

        return self::bool_operation(
            context,
            lhs,
            rhs,
            operator,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn const_bool_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let lhs_signed: bool = signatures.0;
    let rhs_signed: bool = signatures.1;

    if lhs.is_int_value() && rhs.is_int_value() {
        let lhs: IntValue = lhs.into_int_value();
        let rhs: IntValue = rhs.into_int_value();

        let (lhs, rhs) = compiler::generation::cast::const_integer_together(lhs, rhs, signatures);

        return match operator {
            op if op.is_logical_operator() => lhs
                .const_int_compare(predicates::integer(operator, lhs_signed, rhs_signed), rhs)
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return lhs.const_and(rhs).into();
                }

                if let TokenType::Or = op {
                    return lhs.const_or(rhs).into();
                }

                self::codegen_abort(
                    "Cannot perform constant boolean binary operation without a valid gate.",
                );
            }
            _ => {
                self::codegen_abort(
                    "Cannot perform constant boolean binary operation without a valid operator.",
                );
            }
        };
    }

    if lhs.is_float_value() && rhs.is_float_value() {
        let lhs: FloatValue = lhs.into_float_value();
        let rhs: FloatValue = rhs.into_float_value();

        let (lhs, rhs) = compiler::generation::cast::const_float_together(lhs, rhs);

        return match operator {
            op if op.is_logical_operator() => {
                lhs.const_compare(predicates::float(operator), rhs).into()
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform constant boolean binary operation without two float values.",
                );
            }
        };
    }

    if lhs.is_pointer_value() && rhs.is_pointer_value() {
        let lhs: PointerValue = lhs.into_pointer_value();
        let rhs: PointerValue = rhs.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => match op {
                TokenType::EqEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() == rhs.is_null()) as u64, false)
                    .into(),

                TokenType::BangEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() != rhs.is_null()) as u64, false)
                    .into(),

                _ => llvm_context.bool_type().const_zero().into(),
            },

            _ => {
                self::codegen_abort(
                    "Cannot perform pointer binary operation without a valid operator.",
                );
            }
        };
    }

    self::codegen_abort(
        "Cannot perform constant boolean binary operation without two integer values.",
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
        TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::And
        | TokenType::Or,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return self::const_bool_operation(
            context,
            lhs,
            rhs,
            operator,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant boolean binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
