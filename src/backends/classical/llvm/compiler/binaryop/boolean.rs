use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::abort;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::value;

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
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let left_signed: bool = signatures.0;
    let right_signed: bool = signatures.1;

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform boolean binary operation.");
    };

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = compiler::generation::cast::integer_together(context, left, right);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, left_signed, right_signed),
                    left,
                    right,
                    "",
                )
                .unwrap_or_else(cintgen_abort)
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return llvm_builder
                        .build_and(left, right, "")
                        .unwrap_or_else(cintgen_abort)
                        .into();
                }

                if let TokenType::Or = op {
                    return llvm_builder
                        .build_or(left, right, "")
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

    if left.is_float_value() && right.is_float_value() {
        let left: FloatValue = left.into_float_value();
        let right: FloatValue = right.into_float_value();

        let (left, right) = compiler::generation::cast::float_together(context, left, right);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_float_compare(predicates::float(operator), left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform boolean binary operation without two float values.",
                );
            }
        };
    }

    if left.is_pointer_value() && right.is_pointer_value() {
        let lhs: PointerValue = left.into_pointer_value();
        let rhs: PointerValue = right.into_pointer_value();

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

        let left: BasicValueEnum = value::compile(context, binary.0, cast);
        let right: BasicValueEnum = value::compile(context, binary.2, cast);

        return self::bool_operation(
            context,
            left,
            right,
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

    let left_signed: bool = signatures.0;
    let right_signed: bool = signatures.1;

    if lhs.is_int_value() && rhs.is_int_value() {
        let left: IntValue = lhs.into_int_value();
        let right: IntValue = rhs.into_int_value();

        let (left, right) =
            compiler::generation::cast::const_integer_together(left, right, signatures);

        return match operator {
            op if op.is_logical_operator() => left
                .const_int_compare(
                    predicates::integer(operator, left_signed, right_signed),
                    right,
                )
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return left.const_and(right).into();
                }

                if let TokenType::Or = op {
                    return left.const_or(right).into();
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
        let left: FloatValue = lhs.into_float_value();
        let right: FloatValue = rhs.into_float_value();

        let (left, right) = compiler::generation::cast::const_float_together(left, right);

        return match operator {
            op if op.is_logical_operator() => left
                .const_compare(predicates::float(operator), right)
                .into(),

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
