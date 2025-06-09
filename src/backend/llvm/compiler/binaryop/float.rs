use crate::{
    backend::llvm::compiler::{predicates, valuegen::CompileChanges},
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
    builder: &Builder<'ctx>,
    mut left: FloatValue<'ctx>,
    mut right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    match operator {
        TokenType::Plus => builder.build_float_add(left, right, "").unwrap().into(),
        TokenType::Minus => builder.build_float_sub(left, right, "").unwrap().into(),
        TokenType::Star => builder.build_float_mul(left, right, "").unwrap().into(),
        TokenType::Slash => builder.build_float_div(left, right, "").unwrap().into(),

        op if op.is_logical_type() => {
            if left.get_type() != right.get_type() {
                left = builder
                    .build_float_cast(left, right.get_type(), "")
                    .unwrap()
            }

            if right.get_type() != left.get_type() {
                right = builder
                    .build_float_cast(right, left.get_type(), "")
                    .unwrap()
            }

            builder
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
    let llvm_builder: &Builder = context.get_llvm_builder();

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
            llvm_builder,
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
