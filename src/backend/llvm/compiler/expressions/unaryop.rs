use std::fmt::Display;

use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::memory::SymbolAllocated;
use crate::backend::llvm::compiler::{cast, typegen, valuegen};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::repr::UnaryOperation;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match unary {
        (TokenType::PlusPlus | TokenType::MinusMinus, _, Ast::Reference { name, kind, .. }) => {
            self::compile_increment_decrement_ref(context, name, unary.0, kind, cast)
        }
        (TokenType::PlusPlus | TokenType::MinusMinus, _, expr) => {
            self::compile_increment_decrement(context, unary.0, expr, cast)
        }

        (TokenType::Bang, _, expr) => self::compile_logical_negation(context, expr, cast),
        (TokenType::Minus, _, expr) => self::compile_arithmetic_negation(context, expr, cast),

        _ => {
            self::codegen_abort("Unsupported unary operation pattern encountered.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_increment_decrement_ref<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    operator: &TokenType,
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let symbol: SymbolAllocated = context.get_table().get_symbol(name);

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => match llvm_builder.build_int_nsw_add(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        self::codegen_abort("Failed to compile an incrementer.");
                        self::compile_null_ptr(context)
                    }
                },
                TokenType::MinusMinus => match llvm_builder.build_int_nsw_sub(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        self::codegen_abort("Failed to compile a decrementer.");
                        self::compile_null_ptr(context)
                    }
                },
                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to reference increment and decrement in unary operation.",
                    );
                    self::compile_null_ptr(context)
                }
            };

            let result: BasicValueEnum =
                cast::try_cast(context, cast, kind, result).unwrap_or(result);

            symbol.store(context, result);

            result
        }
        _ => {
            let float: FloatValue = symbol.load(context).into_float_value();

            let modifier: FloatValue =
                typegen::float_to_llvm_type(llvm_context, kind).const_float(1.0);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap()
                    .into(),
                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap()
                    .into(),

                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to reference increment and decrement in unary operation.",
                    );
                    self::compile_null_ptr(context)
                }
            };

            let result: BasicValueEnum =
                cast::try_cast(context, cast, kind, result).unwrap_or(result);

            symbol.store(context, result);

            result
        }
    }
}

fn compile_increment_decrement<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    operator: &TokenType,
    expression: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = valuegen::compile(context, expression, cast);
    let kind: &Type = expression.get_type_unwrapped();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            let modifier: IntValue = int.get_type().const_int(1, false);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => match llvm_builder.build_int_nsw_add(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        self::codegen_abort("Failed to compile an incrementer.");
                        self::compile_null_ptr(context)
                    }
                },
                TokenType::MinusMinus => match llvm_builder.build_int_nsw_sub(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        self::codegen_abort("Failed to compile a decrementer.");
                        self::compile_null_ptr(context)
                    }
                },

                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );
                    self::compile_null_ptr(context)
                }
            };

            cast::try_cast(context, cast, kind, result).unwrap_or(result)
        }
        _ => {
            let float: FloatValue = value.into_float_value();

            let modifier: FloatValue = float.get_type().const_float(1.0);

            let result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap()
                    .into(),

                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap()
                    .into(),

                _ => {
                    self::codegen_abort(
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );
                    self::compile_null_ptr(context)
                }
            };

            cast::try_cast(context, cast, kind, result).unwrap_or(result)
        }
    }
}

fn compile_logical_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = valuegen::compile(context, expr, cast);
    let kind: &Type = expr.get_type_unwrapped();

    match kind {
        kind if kind.is_bool_type() => {
            let int: IntValue = value.into_int_value();

            if let Ok(result) = llvm_builder.build_not(int, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast, kind, result).unwrap_or(result);
            }

            int.into()
        }

        _ => {
            self::codegen_abort("Cannot perform a logical negation.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_arithmetic_negation<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: BasicValueEnum = valuegen::compile(context, expr, cast);
    let kind: &Type = expr.get_type_unwrapped();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            if let Ok(result) = llvm_builder.build_int_neg(int, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast, kind, result).unwrap_or(result);
            }

            int.into()
        }

        _ => {
            let float: FloatValue = value.into_float_value();

            if let Ok(result) = llvm_builder.build_float_neg(float, "") {
                let result: BasicValueEnum = result.into();

                return cast::try_cast(context, cast, kind, result).unwrap_or(result);
            }

            float.into()
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
