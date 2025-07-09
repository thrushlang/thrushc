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
            logging::log(
                logging::LoggingType::Error,
                "Unsupported unary operation pattern encountered.",
            );
            unreachable!()
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

    let symbol: SymbolAllocated = context.get_symbol(name);

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue =
                typegen::integer_to_llvm_type(llvm_context, kind).const_int(1, false);

            let mut result: BasicValueEnum = match operator {
                TokenType::PlusPlus => match llvm_builder.build_int_nsw_add(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        logging::log(
                            logging::LoggingType::Bug,
                            "Failed to compile an incrementer.",
                        );
                        unreachable!()
                    }
                },
                TokenType::MinusMinus => match llvm_builder.build_int_nsw_sub(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        logging::log(
                            logging::LoggingType::Bug,
                            "Failed to compile a decrementer.",
                        );
                        unreachable!()
                    }
                },
                _ => {
                    logging::log(
                        LoggingType::Bug,
                        "Unknown operator compared to reference increment and decrement in unary operation.",
                    );

                    unreachable!()
                }
            };

            symbol.store(context, result);

            if let Some(cast) = cast {
                if let Some(casted_int) = cast::integer(context, cast, kind, result) {
                    result = casted_int;
                }
            }

            result
        }
        _ => {
            let float: FloatValue = symbol.load(context).into_float_value();

            let modifier: FloatValue =
                typegen::type_float_to_llvm_float_type(llvm_context, kind).const_float(1.0);

            let mut result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap()
                    .into(),
                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap()
                    .into(),
                _ => unreachable!(),
            };

            if let Some(cast) = cast {
                if let Some(casted_float) = cast::float(context, cast, kind, result) {
                    result = casted_float;
                }
            }

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
    let llvm_context: &Context = context.get_llvm_context();

    let value: BasicValueEnum = valuegen::compile(context, expression, cast);
    let kind: &Type = expression.get_type_unwrapped();

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = value.into_int_value();

            let modifier: IntValue =
                typegen::integer_to_llvm_type(llvm_context, kind).const_int(1, false);

            let mut result: BasicValueEnum = match operator {
                TokenType::PlusPlus => match llvm_builder.build_int_nsw_add(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        logging::log(
                            logging::LoggingType::Bug,
                            "Failed to compile an incrementer.",
                        );

                        unreachable!()
                    }
                },
                TokenType::MinusMinus => match llvm_builder.build_int_nsw_sub(int, modifier, "") {
                    Ok(result) => result.into(),
                    Err(_) => {
                        logging::log(
                            logging::LoggingType::Bug,
                            "Failed to compile a decrementer.",
                        );
                        unreachable!()
                    }
                },

                _ => {
                    logging::log(
                        LoggingType::Bug,
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );

                    unreachable!()
                }
            };

            if let Some(cast) = cast {
                if let Some(casted_int) = cast::integer(context, cast, kind, result) {
                    result = casted_int;
                }
            }

            result
        }
        _ => {
            let float: FloatValue = value.into_float_value();

            let modifier: FloatValue =
                typegen::type_float_to_llvm_float_type(llvm_context, kind).const_float(1.0);

            let mut result: BasicValueEnum = match operator {
                TokenType::PlusPlus => llvm_builder
                    .build_float_add(float, modifier, "")
                    .unwrap()
                    .into(),

                TokenType::MinusMinus => llvm_builder
                    .build_float_sub(float, modifier, "")
                    .unwrap()
                    .into(),

                _ => {
                    logging::log(
                        LoggingType::Bug,
                        "Unknown operator compared to increment and decrement in unary operation.",
                    );

                    unreachable!()
                }
            };

            if let Some(cast) = cast {
                if let Some(casted_float) = cast::float(context, cast, kind, result) {
                    result = casted_float;
                }
            }

            result
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
                let mut result: BasicValueEnum = result.into();

                if let Some(cast) = cast {
                    if let Some(casted_int) = cast::integer(context, cast, kind, result) {
                        result = casted_int;
                    }
                }

                return result;
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
                let mut result: BasicValueEnum = result.into();

                if let Some(cast) = cast {
                    if let Some(casted_int) = cast::integer(context, cast, kind, result) {
                        result = casted_int;
                    }
                }

                return result;
            }

            int.into()
        }

        _ => {
            let float: FloatValue = value.into_float_value();

            if let Ok(result) = llvm_builder.build_float_neg(float, "") {
                let mut result: BasicValueEnum = result.into();

                if let Some(cast) = cast {
                    if let Some(casted_float) = cast::float(context, cast, kind, result) {
                        result = casted_float;
                    }
                }

                return result;
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
