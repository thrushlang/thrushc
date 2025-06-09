use crate::{
    backend::llvm::compiler::{cast, predicates, valuegen::CompileChanges},
    core::console::logging::{self, LoggingType},
    frontend::{
        lexer::tokentype::TokenType,
        types::{lexer::ThrushType, representations::BinaryOperation},
    },
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

        op if op.is_logical_type() => {
            let (left, right) = cast::float_together(context, left, right);

            llvm_builder
                .build_float_compare(predicates::float(operator), left, right, "")
                .unwrap()
                .into()
        }

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
    cast: &ThrushType,
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
            valuegen::compile(context, binary.0, cast, CompileChanges::new(false, true));

        let right_compiled: BasicValueEnum =
            valuegen::compile(context, binary.2, cast, CompileChanges::new(false, true));

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
