use {
    super::super::{context::LLVMCodeGenContext, valuegen},
    crate::{
        backend::llvm::compiler::{cast, constgen, predicates},
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType,
            types::{lexer::Type, parser::repr::BinaryOperation},
        },
    },
    inkwell::{
        AddressSpace,
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

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = cast::integer_together(context, left, right);

        return match operator {
            TokenType::Plus => llvm_builder
                .build_int_nsw_add(left, right, "")
                .unwrap()
                .into(),
            TokenType::Minus => llvm_builder
                .build_int_nsw_sub(left, right, "")
                .unwrap()
                .into(),
            TokenType::Star => llvm_builder
                .build_int_nsw_mul(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if signatures.0 || signatures.1 => llvm_builder
                .build_int_signed_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::Slash if !signatures.0 && !signatures.1 => llvm_builder
                .build_int_unsigned_div(left, right, "")
                .unwrap()
                .into(),
            TokenType::LShift => llvm_builder
                .build_left_shift(left, right, "")
                .unwrap()
                .into(),
            TokenType::RShift => llvm_builder
                .build_right_shift(left, right, signatures.0 || signatures.1, "")
                .unwrap()
                .into(),

            op if op.is_logical_type() => llvm_builder
                .build_int_compare(
                    predicates::integer(operator, signatures.0, signatures.1),
                    left,
                    right,
                    "",
                )
                .unwrap()
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
                self::compile_null_ptr(context)
            }

            _ => {
                self::codegen_abort(
                    "Cannot perform integer binary operation without a valid operator.",
                );
                self::compile_null_ptr(context)
            }
        };
    }

    self::codegen_abort("Cannot perform integer binary operation without integer values.");
    self::compile_null_ptr(context)
}

pub fn integer_binaryop<'ctx>(
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
        | TokenType::GreaterEq
        | TokenType::LShift
        | TokenType::RShift
        | TokenType::And
        | TokenType::Or,
        _,
    ) = binary
    {
        let operator: &TokenType = binary.1;

        let left: BasicValueEnum = valuegen::compile(context, binary.0, cast_type);
        let right: BasicValueEnum = valuegen::compile(context, binary.2, cast_type);

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

    self::compile_null_ptr(context)
}

fn const_int_operation<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
    signatures: (bool, bool),
    operator: &TokenType,
) -> BasicValueEnum<'ctx> {
    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

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

            op if op.is_logical_type() => left
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

pub fn const_integer_binaryop<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    binary: BinaryOperation<'ctx>,
    kind: &Type,
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

        let left: BasicValueEnum = constgen::compile(context, binary.0, kind);
        let right: BasicValueEnum = constgen::compile(context, binary.2, kind);

        return const_int_operation(
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
