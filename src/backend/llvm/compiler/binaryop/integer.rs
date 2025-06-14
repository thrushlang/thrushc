use {
    super::super::{context::LLVMCodeGenContext, valuegen},
    crate::{
        backend::llvm::compiler::{cast, predicates},
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType,
            types::{lexer::ThrushType, representations::BinaryOperation},
        },
    },
    inkwell::{
        builder::Builder,
        values::{BasicValueEnum, IntValue},
    },
};

pub fn int_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = cast::integer_together(context, left, right);

        return match operator {
            TokenType::Plus => llvm_builder
                .build_int_nsw_add(left, right, "")
                .unwrap()
                .into(),
            TokenType::Minus => llvm_builder
                .build_int_nsw_sub(left, right, "")
                .unwrap()
                .into(),
            TokenType::Star => llvm_builder
                .build_int_nsw_mul(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::LShift => llvm_builder
                .build_left_shift(left, right, "")
                .unwrap()
                .into(),
            TokenType::RShift => llvm_builder
                .build_right_shift(left, right, signatures.0 || signatures.1, "")
                .unwrap()
                .into(),

            op if op.is_logical_type() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, signatures.0, signatures.1),
                    left,
                    right,
                    "",
                )
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
                    "Unable to perform integer binary operation without valid logical gate.",
                );

                unreachable!()
            }

            _ => {
                logging::log(
                    LoggingType::Bug,
                    "Unable to perform integer binary operation without valid operator.",
                );

                unreachable!()
            }
        };
    }

    logging::log(
        LoggingType::Bug,
        "Unable to perform integer binary operation without int values.",
    );

    unreachable!()
}

pub fn integer_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast_type: Option<&ThrushType>,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast_type);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast_type);

        return int_operation(
            context,
            left,
            right,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
            operator,
        );
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a integer binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
