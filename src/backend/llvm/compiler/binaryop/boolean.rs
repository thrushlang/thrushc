use {
    crate::{
        backend::llvm::compiler::{
            cast,
            context::LLVMCodeGenContext,
            predicates,
            valuegen::{self},
        },
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType, types::parser::repr::BinaryOperation,
            typesystem::types::Type,
        },
    },
    inkwell::{
        builder::Builder,
        values::{BasicValueEnum, FloatValue, IntValue},
    },
    std::fmt::Display,
};

pub fn bool_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    operator: &TokenType,
    signatures: (bool, bool),
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let left_signed: bool = signatures.0;
    let right_signed: bool = signatures.1;

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform boolean binary operation.");
    };

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = cast::integer_together(context, left, right);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, left_signed, right_signed),
                    left,
                    right,
                    "",
                )
                .unwrap_or_else(cintgen_abort)
                .into(),

            op if op.is_logical_gate() => {
                if let TokenType::And = op {
                    return llvm_builder
                        .build_and(left, right, "")
                        .unwrap_or_else(cintgen_abort)
                        .into();
                }

                if let TokenType::Or = op {
                    return llvm_builder
                        .build_or(left, right, "")
                        .unwrap_or_else(cintgen_abort)
                        .into();
                }

                self::codegen_abort(
                    "Cannot perform boolean binary operation without a valid gate.",
                );
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform boolean binary operation without a valid operator.",
                );
            }
        };
    }

    if left.is_float_value() && right.is_float_value() {
        let left: FloatValue = left.into_float_value();
        let right: FloatValue = right.into_float_value();

        let (left, right) = cast::float_together(context, left, right);

        return match operator {
            op if op.is_logical_operator() => llvm_builder
                .build_float_compare(predicates::float(operator), left, right, "")
                .unwrap_or_else(cintgen_abort)
                .into(),

            _ => {
                self::codegen_abort(
                    "Cannot perform boolean binary operation without two float values.",
                );
            }
        };
    }

    self::codegen_abort("Cannot perform boolean binary operation without two integer values.");
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    cast: Option<&Type>,
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

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast);

        return self::bool_operation(
            context,
            left,
            right,
            operator,
            (
                binary.0.get_type_unwrapped().is_signed_integer_type(),
                binary.2.get_type_unwrapped().is_signed_integer_type(),
            ),
        );
    }

    self::codegen_abort(format!(
        "Cannot perform process a boolean binary operation '{} {} {}'.",
        binary.0, binary.1, binary.2
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
