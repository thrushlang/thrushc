use crate::backends::classical::llvm::compiler::cast;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;

use inkwell::values::BasicValueEnum;
use inkwell::values::IntValue;

use std::fmt::Display;

fn const_int_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    if lhs.is_int_value() && rhs.is_int_value() {
        let left: IntValue = lhs.into_int_value();
        let right: IntValue = rhs.into_int_value();

        let (left, right) = cast::const_integer_together(left, right, signatures);

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

                self::compile_null_ptr(context)
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform constant integer binary operation without a valid operator.",
                );
                self::compile_null_ptr(context)
            }
        };
    }

    self::codegen_abort("Cannot perform constant integer binary operation without integer values.");
    self::compile_null_ptr(context)
}

pub fn compile<'ctx>(
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
        | TokenType::Or,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return self::const_int_operation(
            context,
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
