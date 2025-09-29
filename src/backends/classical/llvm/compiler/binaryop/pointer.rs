use crate::backends::classical::llvm::compiler::abort;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::value;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;

use std::fmt::Display;
use std::path::PathBuf;

use inkwell::context::Context;
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
    let span: Span = binary.3;

    if let (_, TokenType::EqEq | TokenType::BangEq | TokenType::Xor | TokenType::Bor, ..) = binary {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = value::compile(context, binary.0, None);
        let right: BasicValueEnum = value::compile(context, binary.2, None);

        return ptr_operation(context, left, right, operator);
    }

    abort::abort_codegen(
        context,
        "Failed to compile pointer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

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

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
) -> BasicValueEnum<'ctx> {
    let span: Span = binary.3;

    if let (_, TokenType::EqEq | TokenType::BangEq, ..) = binary {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = value::compile(context, binary.0, None);
        let right: BasicValueEnum = value::compile(context, binary.2, None);

        return const_ptr_operation(context, left, right, operator);
    }

    abort::abort_codegen(
        context,
        "Failed to compile constant pointer binary operation!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
