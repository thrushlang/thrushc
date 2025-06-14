use {
    crate::{
        backend::llvm::compiler::{
            cast,
            context::LLVMCodeGenContext,
            predicates,
            valuegen::{self},
        },
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType,
            types::{lexer::ThrushType, representations::BinaryOperation},
        },
    },
    inkwell::{
        builder::Builder,
        values::{BasicValueEnum, FloatValue, IntValue},
    },
};

pub fn bool_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = cast::integer_together(context, left, right);

        return match operator {
            op if op.is_logical_type() => llvm_builder
                .build_int_compare(predicates::integer(operator, false, false), left, right, "")
                .unwrap()
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return llvm_builder.build_and(left, right, "").unwrap().into();
                }

                if let TokenType::Or = op {
                    return llvm_builder.build_or(left, right, "").unwrap().into();
                }

                logging::log(
                    LoggingType::Bug,
                    "Unable to perform boolean binary operation without valid gate.",
                );

                unreachable!()
            }
            _ => unreachable!(),
        };
    }

    if left.is_float_value() && right.is_float_value() {
        let left: FloatValue = left.into_float_value();
        let right: FloatValue = right.into_float_value();

        let (left, right) = cast::float_together(context, left, right);

        return match operator {
            op if op.is_logical_type() => llvm_builder
                .build_float_compare(predicates::float(operator), left, right, "")
                .unwrap()
                .into(),

            _ => unreachable!(),
        };
    }

    logging::log(
        LoggingType::Bug,
        "Unable to perform boolean binary operation without two int values.",
    );

    unreachable!()
}

pub fn bool_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
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
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast_type);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast_type);

        return self::bool_operation(context, left, right, operator);
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a boolean binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
