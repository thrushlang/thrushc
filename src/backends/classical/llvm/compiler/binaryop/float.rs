use std::{fmt::Display, path::PathBuf};

use crate::{
    backends::classical::llvm::compiler::{self, abort, codegen, constgen, predicates},
    core::console::logging::{self, LoggingType},
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        types::parser::repr::BinaryOperation,
        typesystem::types::Type,
    },
};

use super::super::context::LLVMCodeGenContext;

use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, FloatValue},
};

pub fn float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    lhs: FloatValue<'ctx>,
    rhs: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let (lhs, rhs) = compiler::generation::cast::float_together(context, lhs, rhs);

    let cfloatgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
    };

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
    };

    match operator {
        TokenType::Plus => llvm_builder
            .build_float_add(lhs, rhs, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Minus => llvm_builder
            .build_float_sub(lhs, rhs, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Star => llvm_builder
            .build_float_mul(lhs, rhs, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Slash => llvm_builder
            .build_float_div(lhs, rhs, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),

        op if op.is_logical_operator() => llvm_builder
            .build_float_compare(predicates::float(operator), lhs, rhs, "")
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

        let lhs: BasicValueEnum = codegen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = codegen::compile(context, binary.2, cast);

        return float_operation(
            context,
            lhs.into_float_value(),
            rhs.into_float_value(),
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
    lhs: FloatValue<'ctx>,
    rhs: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let (lhs, rhs) = compiler::generation::cast::const_float_together(lhs, rhs);

    match operator {
        TokenType::Plus => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number + rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Minus => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number - rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Star => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number * rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        TokenType::Slash => {
            if let Some(lhs_constant) = lhs.get_constant() {
                if let Some(rhs_constant) = rhs.get_constant() {
                    let lhs_number: f64 = lhs_constant.0;
                    let rhs_number: f64 = rhs_constant.0;

                    return lhs.get_type().const_float(lhs_number / rhs_number).into();
                }
            }

            lhs.get_type().const_zero().into()
        }

        op if op.is_logical_operator() => {
            lhs.const_compare(predicates::float(operator), rhs).into()
        }

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
