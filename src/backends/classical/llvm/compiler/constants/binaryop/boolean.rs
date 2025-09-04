use crate::backends::classical::llvm::compiler::cast;
use crate::backends::classical::llvm::compiler::constgen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::predicates;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::types::parser::repr::BinaryOperation;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;
use inkwell::values::IntValue;

use std::fmt::Display;

pub fn const_bool_operation<'ctx>(
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
) -> BasicValueEnum<'ctx> {
    let left_signed: bool = signatures.0;
    let right_signed: bool = signatures.1;

    if lhs.is_int_value() && rhs.is_int_value() {
        let left: IntValue = lhs.into_int_value();
        let right: IntValue = rhs.into_int_value();

        let (left, right) = cast::const_integer_together(left, right, signatures);

        return match operator {
            op if op.is_logical_operator() => left
                .const_int_compare(
                    predicates::integer(operator, left_signed, right_signed),
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
                    "Cannot perform constant boolean binary operation without a valid gate.",
                );
            }
            _ => {
                self::codegen_abort(
                    "Cannot perform constant boolean binary operation without a valid operator.",
                );
            }
        };
    }

    if lhs.is_float_value() && rhs.is_float_value() {
        let left: FloatValue = lhs.into_float_value();
        let right: FloatValue = rhs.into_float_value();

        let (left, right) = cast::const_float_together(left, right);

        return match operator {
            op if op.is_logical_operator() => left
                .const_compare(predicates::float(operator), right)
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform constant boolean binary operation without two float values.",
                );
            }
        };
    }

    self::codegen_abort(
        "Cannot perform constant boolean binary operation without two integer values.",
    );
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    if let (
        _,
        TokenType::BangEq
        | TokenType::EqEq
        | TokenType::LessEq
        | TokenType::Less
        | TokenType::Greater
        | TokenType::GreaterEq
        | TokenType::And
        | TokenType::Or,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let lhs: BasicValueEnum = constgen::compile(context, binary.0, cast);
        let rhs: BasicValueEnum = constgen::compile(context, binary.2, cast);

        return self::const_bool_operation(
            lhs,
            rhs,
            operator,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
        );
    }

    self::codegen_abort(format!(
        "Cannot perform process a constant boolean binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
