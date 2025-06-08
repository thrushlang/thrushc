use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        predicates,
        valuegen::{self, ExpressionModificator},
    },
    core::console::logging::{self, LoggingType},
    frontend::{
        lexer::tokentype::TokenType,
        types::{lexer::ThrushType, representations::BinaryOperation},
    },
};

pub fn ptr_operation<'ctx>(
    builder: &Builder<'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    if left.is_pointer_value() && right.is_pointer_value() {
        let left: PointerValue = left.into_pointer_value();
        let right: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_type() => builder
                .build_int_compare(predicates::pointer(operator), left, right, "")
                .unwrap()
                .into(),

            _ => {
                logging::log(
                    LoggingType::Bug,
                    "Unable to perform pointer binary operation without valid operator.",
                );

                unreachable!()
            }
        };
    }

    logging::log(
        LoggingType::Bug,
        "Unable to perform pointer binary operation without two pointer values.",
    );

    unreachable!()
}

pub fn ptr_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    target_type: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let (_, TokenType::EqEq | TokenType::BangEq, _) = binary {
        let left_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.0,
            target_type,
            ExpressionModificator::new(false, true),
        );

        let right_compiled: BasicValueEnum = valuegen::compile(
            context,
            binary.2,
            target_type,
            ExpressionModificator::new(false, true),
        );

        return ptr_operation(llvm_builder, left_compiled, right_compiled, binary.1);
    }

    logging::log(
        LoggingType::Panic,
        &format!(
            "Could not process a pointer binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
