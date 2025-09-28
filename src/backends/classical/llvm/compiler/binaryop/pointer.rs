use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::value;

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

    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    if lhs.is_pointer_value() && rhs.is_pointer_value() {
        let lhs: PointerValue = lhs.into_pointer_value();
        let rhs: PointerValue = rhs.into_pointer_value();

        return match operator {
            TokenType::Xor => llvm_builder
                .build_xor(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    self::codegen_abort("Cannot perform pointer binary operation.");
                })
                .into(),
            TokenType::Bor => llvm_builder
                .build_or(lhs, rhs, "")
                .unwrap_or_else(|_| {
                    self::codegen_abort("Cannot perform pointer binary operation.");
                })
                .into(),

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
    if let (_, TokenType::EqEq | TokenType::BangEq | TokenType::Xor | TokenType::Bor, _) = binary {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = value::compile(context, binary.0, None);
        let right: BasicValueEnum = value::compile(context, binary.2, None);

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
