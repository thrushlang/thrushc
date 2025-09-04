use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::valuegen;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;

use std::fmt::Display;

use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, PointerValue},
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
