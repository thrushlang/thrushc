use std::fmt::Display;

use crate::{
    backend::llvm::compiler::{cast, predicates},
    core::console::logging::{self, LoggingType},
    frontend::{
        lexer::tokentype::TokenType, types::parser::repr::BinaryOperation, typesystem::types::Type,
    },
};

use super::super::{context::LLVMCodeGenContext, valuegen};

use inkwell::{
    AddressSpace,
    builder::Builder,
    values::{BasicValueEnum, FloatValue},
};

pub fn float_operation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    left: FloatValue<'ctx>,
    right: FloatValue<'ctx>,
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let (left, right) = cast::float_together(context, left, right);

    let cfloatgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
        unreachable!()
    };

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
        unreachable!()
    };

    match operator {
        TokenType::Plus => llvm_builder
            .build_float_add(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Minus => llvm_builder
            .build_float_sub(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Star => llvm_builder
            .build_float_mul(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),
        TokenType::Slash => llvm_builder
            .build_float_div(left, right, "")
            .unwrap_or_else(cfloatgen_abort)
            .into(),

        op if op.is_logical_operator() => llvm_builder
            .build_float_compare(predicates::float(operator), left, right, "")
            .unwrap_or_else(cintgen_abort)
            .into(),

        _ => {
            self::codegen_abort("Cannot perform float binary operation without a valid operator.");
            self::compile_null_ptr(context)
        }
    }
}

pub fn float_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast_type: Option<&Type>,
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
        | TokenType::GreaterEq,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast_type);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast_type);

        return float_operation(
            context,
            left.into_float_value(),
            right.into_float_value(),
            operator,
        );
    }

    self::codegen_abort(format!(
        "Cannot perform process a float binary operation '{} {} {}'.",
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
