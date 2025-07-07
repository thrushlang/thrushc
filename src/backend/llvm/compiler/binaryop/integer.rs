use {
    super::super::{context::LLVMCodeGenContext, valuegen},
    crate::{
        backend::llvm::compiler::{cast, predicates},
        core::console::logging::{self, LoggingType},
        frontend::{
            lexer::tokentype::TokenType, types::parser::repr::BinaryOperation,
            typesystem::types::Type,
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

    let cintgen_abort = |_| {
        self::codegen_abort("Cannot perform integer binary operation.");
        unreachable!()
    };

    if left.is_int_value() && right.is_int_value() {
        let left: IntValue = left.into_int_value();
        let right: IntValue = right.into_int_value();

        let (left, right) = cast::integer_together(context, left, right);

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
