use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        predicates,
        valuegen::{self},
    },
    core::console::logging::{self, LoggingType},
    frontend::{lexer::tokentype::TokenType, types::parser::repr::BinaryOperation},
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
                    LoggingType::BackendPanic,
                    "Cannot perform pointer binary operation without a valid operator.",
                );

                unreachable!()
            }
        };
    }

    logging::log(
        LoggingType::BackendPanic,
        "Cannot perform pointer binary operation without two pointers.",
    );

    unreachable!()
}

pub fn ptr_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let (_, TokenType::EqEq | TokenType::BangEq, _) = binary {
        let operator: &TokenType = binary.1;
        let left: BasicValueEnum = valuegen::compile(context, binary.0, None);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, None);

        return ptr_operation(llvm_builder, left, right, operator);
    }

    logging::log(
        LoggingType::BackendPanic,
        &format!(
            "Cannot perform a pointer binary operation '{} {} {}'.",
            binary.0, binary.1, binary.2
        ),
    );

    unreachable!()
}
