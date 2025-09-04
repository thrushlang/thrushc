use std::fmt::Display;

use crate::{
    backends::classical::llvm::compiler::{cast, predicates},
    core::console::logging::{self, LoggingType},
    frontends::classical::{
        lexer::tokentype::TokenType, types::parser::repr::BinaryOperation, typesystem::types::Type,
    },
};

use super::super::{context::LLVMCodeGenContext, valuegen};

use inkwell::{
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
    };

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform float binary operation.");
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
        }
    }
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: Option<&Type>,
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

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast);

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
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
