use crate::{
    backend::llvm::compiler::{cast, predicates, valuegen::CompileChanges},
    core::console::logging::{self, LoggingType},
    frontend::{lexer::tokentype::TokenType, types::representations::BinaryOperation},
};

use super::super::{context::LLVMCodeGenContext, valuegen};

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

    let (left, right) = cast::float_together(context, left, right);

    match operator {
        TokenType::Plus => llvm_builder
            .build_float_add(left, right, "")
            .unwrap()
            .into(),
        TokenType::Minus => llvm_builder
            .build_float_sub(left, right, "")
            .unwrap()
            .into(),
        TokenType::Star => llvm_builder
            .build_float_mul(left, right, "")
            .unwrap()
            .into(),
        TokenType::Slash => llvm_builder
            .build_float_div(left, right, "")
            .unwrap()
            .into(),

        op if op.is_logical_type() => llvm_builder
            .build_float_compare(predicates::float(operator), left, right, "")
            .unwrap()
            .into(),

        _ => {
            logging::log(
                LoggingType::Bug,
                "Unable to perform float binary operation without valid operator.",
            );

            unreachable!()
        }
    }
}

pub fn float_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
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
        let left_compiled: BasicValueEnum =
            valuegen::compile(context, binary.0, CompileChanges::new(false));

        let right_compiled: BasicValueEnum =
            valuegen::compile(context, binary.2, CompileChanges::new(false));

        return float_operation(
            context,
            left_compiled.into_float_value(),
            right_compiled.into_float_value(),
            binary.1,
        );
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a float binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
