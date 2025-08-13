use std::fmt::Display;

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
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if left.is_pointer_value() && right.is_pointer_value() {
        let lhs: PointerValue = left.into_pointer_value();
        let rhs: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(predicates::pointer(operator), lhs, rhs, "")
                .unwrap_or_else(|_| {
                    self::codegen_abort("Cannot perform pointer binary operation.");
                })
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform pointer binary operation without a valid operator.",
                );
            }
        };
    }

    self::codegen_abort("Cannot perform pointer binary operation without two pointers.");
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (_, TokenType::EqEq | TokenType::BangEq, _) = binary {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = valuegen::compile(context, binary.0, None);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, None);

        return ptr_operation(context, left, right, operator);
    }

    self::codegen_abort(format!(
        "Cannot perform a pointer binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
