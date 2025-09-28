use crate::backends::classical::llvm::compiler;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;
use crate::backends::classical::llvm::compiler::value;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;
use crate::frontends::classical::typesystem::types::Type;

use {
    inkwell::{
        builder::Builder,
        context::Context,
        values::{BasicValueEnum, IntValue},
    },
    std::fmt::Display,
};

fn int_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform integer binary operation.");
    };

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = compiler::generation::cast::integer_together(context, left, right);

        return match operator {
            TokenType::Plus => llvm_builder
                .build_int_nsw_add(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Minus => llvm_builder
                .build_int_nsw_sub(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Star => llvm_builder
                .build_int_nsw_mul(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::LShift => llvm_builder
                .build_left_shift(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::RShift => llvm_builder
                .build_right_shift(left, right, signatures.0 || signatures.1, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Xor => llvm_builder
                .build_xor(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),
            TokenType::Bor => llvm_builder
                .build_or(left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),

            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, signatures.0, signatures.1),
                    left,
                    right,
                    "",
                )
                .unwrap_or_else(cintgen_abort)
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    if let Ok(and) = llvm_builder.build_and(left, right, "") {
                        return and.into();
                    }

                    return llvm_context.bool_type().const_zero().into();
                }

                if let TokenType::Or = op {
                    if let Ok(or) = llvm_builder.build_or(left, right, "") {
                        return or.into();
                    }

                    return llvm_context.bool_type().const_zero().into();
                }

                self::codegen_abort(
                    "Cannot perform integer binary operation without a valid logical gate.",
                );
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform integer binary operation without a valid operator.",
                );
            }
        };
    }

    self::codegen_abort("Cannot perform integer binary operation without integer values.");
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or
        | TokenType::Xor
        | TokenType::Bor,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = value::compile(context, binary.0, cast);
        let right: BasicValueEnum = value::compile(context, binary.2, cast);

        return int_operation(
            context,
            left,
            right,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
            operator,
        );
    }

    self::codegen_abort(format!(
        "Cannot perform integer binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

fn const_int_operation<'ctx>(
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    if lhs.is_int_value() && rhs.is_int_value() {
        let left: IntValue = lhs.into_int_value();
        let right: IntValue = rhs.into_int_value();

        let (left, right) =
            compiler::generation::cast::const_integer_together(left, right, signatures);

        return match operator {
            TokenType::Plus => left.const_nsw_add(right).into(),
            TokenType::Minus => left.const_nsw_sub(right).into(),
            TokenType::Star => left.const_nsw_mul(right).into(),
            TokenType::Slash => {
                if signatures.0 || signatures.1 {
                    if let Some(left_number) = left.get_sign_extended_constant() {
                        if let Some(right_number) = right.get_sign_extended_constant() {
                            return left
                                .get_type()
                                .const_int((left_number / right_number) as u64, true)
                                .into();
                        }
                    }
                }

                if let Some(left_number) = left.get_zero_extended_constant() {
                    if let Some(right_number) = right.get_zero_extended_constant() {
                        return left
                            .get_type()
                            .const_int(left_number / right_number, false)
                            .into();
                    }
                }

                left.get_type().const_zero().into()
            }
            TokenType::LShift => left.const_shl(right).into(),
            TokenType::RShift => left.const_rshr(right).into(),
            TokenType::Xor => left.const_xor(right).into(),
            TokenType::Bor => left.const_or(right).into(),

            op if op.is_logical_operator() => left
                .const_int_compare(
                    predicates::integer(operator, signatures.0, signatures.1),
                    right,
                )
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return left.const_and(right).into();
                }

                if let TokenType::Or = op {
                    return left.const_or(right).into();
                }

                self::codegen_abort(
                    "Cannot perform constant integer binary operation without a valid logical gate.",
                );
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform constant integer binary operation without a valid operator.",
                );
            }
        };
    }

    self::codegen_abort("Cannot perform constant integer binary operation without integer values.");
}

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &Type,
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or
        | TokenType::Xor
        | TokenType::Bor,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return self::const_int_operation(
            lhs,
            rhs,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
            operator,
        );
    }

    self::codegen_abort(format!(
        "Cannot perform constant integer binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
