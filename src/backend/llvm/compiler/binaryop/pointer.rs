use std::fmt::Display;

use inkwell::{
    AddressSpace,
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

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform pointer binary operation.");
        unreachable!()
    };

    if left.is_pointer_value() && right.is_pointer_value() {
        let lhs: PointerValue = left.into_pointer_value();
        let rhs: PointerValue = right.into_pointer_value();

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(predicates::pointer(operator), lhs, rhs, "")
                .unwrap_or_else(cintgen_abort)
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform pointer binary operation without a valid operator.",
                );
                self::compile_null_ptr(context)
            }
        };
    }

    self::codegen_abort("Cannot perform pointer binary operation without two pointers.");
    self::compile_null_ptr(context)
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

    self::compile_null_ptr(context)
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
