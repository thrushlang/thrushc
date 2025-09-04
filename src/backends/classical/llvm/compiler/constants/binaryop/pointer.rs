use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::valuegen;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;

use inkwell::context::Context;

use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;

use std::fmt::Display;

pub fn const_ptr_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    if left.is_pointer_value() && right.is_pointer_value() {
        let lhs: PointerValue = left.into_pointer_value();
        let rhs: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => match op {
                TokenType::EqEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() == rhs.is_null()) as u64, false)
                    .into(),

                TokenType::BangEq => llvm_context
                    .bool_type()
                    .const_int((lhs.is_null() != rhs.is_null()) as u64, false)
                    .into(),

                _ => llvm_context.bool_type().const_zero().into(),
            },

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

        return const_ptr_operation(context, left, right, operator);
    }

    self::codegen_abort(format!(
        "Cannot perform a constant pointer binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
