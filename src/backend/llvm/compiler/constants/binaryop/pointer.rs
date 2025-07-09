use {
    crate::{
        backend::llvm::compiler::{context::LLVMCodeGenContext, valuegen},
        core::console::logging::{self, LoggingType},
        frontend::{lexer::tokentype::TokenType, types::parser::repr::BinaryOperation},
    },
    inkwell::{
        AddressSpace,
        context::Context,
        values::{BasicValueEnum, PointerValue},
    },
    std::fmt::Display,
};

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

        return const_ptr_operation(context, left, right, operator);
    }

    self::codegen_abort(format!(
        "Cannot perform a constant pointer binary operation '{} {} {}'.",
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
