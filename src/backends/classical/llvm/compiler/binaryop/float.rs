use std::{fmt::Display, path::PathBuf};

use crate::{
    backends::classical::llvm::compiler::{self, abort, constgen, predicates},
    core::console::logging::{self, LoggingType},
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        types::parser::repr::BinaryOperation,
        typesystem::types::Type,
    },
};

use super::super::{context::LLVMCodeGenContext, value};

use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, FloatValue},
};

pub fn float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let (left, right) = compiler::generation::cast::float_together(context, left, right);

    let cfloatgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
    };

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
    };

    match operator {
        TokenType::Plus => llvm_builder
            .build_float_add(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Minus => llvm_builder
            .build_float_sub(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Star => llvm_builder
            .build_float_mul(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Slash => llvm_builder
            .build_float_div(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),

        op if op.is_logical_operator() => llvm_builder
            .build_float_compare(predicates::float(operator), left, right, "")
            .unwrap_or_else(cintgen_abort)
            .into(),

        _ => {
            self::codegen_abort("Cannot perform float binary operation without a valid operator.");
        }
    }
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
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = value::compile(context, binary.0, cast);
        let right: BasicValueEnum = value::compile(context, binary.2, cast);

        return float_operation(
            context,
            left.into_float_value(),
            right.into_float_value(),
            operator,
        );
    }

    abort::abort_codegen(
        context,
        "Failed to compile float binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

#[inline]
pub fn const_float_operation<'ctx>(
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let (left, right) = compiler::generation::cast::const_float_together(left, right);

    match operator {
        TokenType::Plus => {
            if let Some(left_constant) = left.get_constant() {
                if let Some(right_constant) = right.get_constant() {
                    let left_number: f64 = left_constant.0;
                    let right_number: f64 = right_constant.0;

                    return left
                        .get_type()
                        .const_float(left_number + right_number)
                        .into();
                }
            }

            left.get_type().const_zero().into()
        }

        TokenType::Minus => {
            if let Some(left_constant) = left.get_constant() {
                if let Some(right_constant) = right.get_constant() {
                    let left_number: f64 = left_constant.0;
                    let right_number: f64 = right_constant.0;

                    return left
                        .get_type()
                        .const_float(left_number - right_number)
                        .into();
                }
            }

            left.get_type().const_zero().into()
        }

        TokenType::Star => {
            if let Some(left_constant) = left.get_constant() {
                if let Some(right_constant) = right.get_constant() {
                    let left_number: f64 = left_constant.0;
                    let right_number: f64 = right_constant.0;

                    return left
                        .get_type()
                        .const_float(left_number * right_number)
                        .into();
                }
            }

            left.get_type().const_zero().into()
        }

        TokenType::Slash => {
            if let Some(left_constant) = left.get_constant() {
                if let Some(right_constant) = right.get_constant() {
                    let left_number: f64 = left_constant.0;
                    let right_number: f64 = right_constant.0;

                    return left
                        .get_type()
                        .const_float(left_number / right_number)
                        .into();
                }
            }

            left.get_type().const_zero().into()
        }

        op if op.is_logical_operator() => left
            .const_compare(predicates::float(operator), right)
            .into(),

        _ => {
            self::codegen_abort(
                "Cannot perform constant float binary operation without a valid operator.",
            );
        }
    }
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
        | TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq,
        ..,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return const_float_operation(lhs.into_float_value(), rhs.into_float_value(), operator);
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant float binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
