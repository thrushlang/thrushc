use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::{self, constgen};

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;
use crate::frontends::classical::typesystem::types::Type;

use std::fmt::Display;

use inkwell::values::{BasicValueEnum, FloatValue};

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

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
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
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return const_float_operation(lhs.into_float_value(), rhs.into_float_value(), operator);
    }

    self::codegen_abort(format!(
        "Cannot perform process a constant float binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
