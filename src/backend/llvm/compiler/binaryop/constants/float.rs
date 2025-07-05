use std::fmt::Display;

use crate::{
    backend::llvm::compiler::{cast, constgen, context::LLVMCodeGenContext, predicates},
    core::console::logging::{self, LoggingType},
    frontend::{
        lexer::tokentype::TokenType,
        types::{lexer::Type, parser::repr::BinaryOperation},
    },
};

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, FloatValue},
};

pub fn const_float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let (left, right) = cast::const_float_together(left, right);

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

            self::compile_null_ptr(context)
        }
    }
}

pub fn const_float_binaryop<'ctx>(
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

        return const_float_operation(
            context,
            lhs.into_float_value(),
            rhs.into_float_value(),
            operator,
        );
    }

    self::codegen_abort(format!(
        "Cannot perform process a constant float binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));

    self::compile_null_ptr(context)
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
