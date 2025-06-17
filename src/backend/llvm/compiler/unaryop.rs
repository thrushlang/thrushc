use crate::backend::llvm::compiler::intgen::{self};
use crate::backend::llvm::compiler::{cast, floatgen};
use crate::core::console::logging;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::parser::stmts::stmt::ThrushStatement;
use crate::frontend::types::representations::UnaryOperation;

use super::{context::LLVMCodeGenContext, memory::SymbolAllocated};

use super::typegen;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn unary_op<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match unary {
        (
            TokenType::PlusPlus | TokenType::MinusMinus,
            _,
            ThrushStatement::Reference { name, kind, .. },
        ) => compile_increment_decrement(context, name, unary.0, kind, cast_type),

        (
            TokenType::Bang,
            _,
            ThrushStatement::Reference {
                name: ref_name,
                kind,
                ..
            },
        ) => compile_logical_negation_ref(context, ref_name, kind, cast_type),

        (TokenType::Minus, _, ThrushStatement::Reference { name, kind, .. }) => {
            compile_arithmetic_negation_ref(context, name, kind, cast_type)
        }

        (TokenType::Bang, _, ThrushStatement::Boolean { value, .. }) => {
            compile_logical_negation_bool(context, *value, cast_type)
        }

        (
            TokenType::Minus,
            _,
            ThrushStatement::Integer {
                kind,
                value,
                signed,
                ..
            },
        ) => compile_arithmetic_negation_int(context, kind, *value, *signed, cast_type),

        (
            TokenType::Minus,
            _,
            ThrushStatement::Float {
                kind,
                value,
                signed,
                ..
            },
        ) => compile_arithmetic_negation_float(context, kind, *value, *signed, cast_type),

        _ => {
            logging::log(
                logging::LoggingType::Error,
                "Unsupported unary operation pattern encountered.",
            );
            unreachable!()
        }
    }
}

fn compile_increment_decrement<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    operator: &TokenType,
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();
    let symbol: SymbolAllocated = context.get_allocated_symbol(name);

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = symbol.load(context).into_int_value();
            let modifier: IntValue =
                typegen::thrush_integer_to_llvm_type(llvm_context, kind).const_int(1, false);

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
                _ => unreachable!(),
            };

            symbol.store(context, result);

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
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

            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                    result = casted_float;
                }
            }

            symbol.store(context, result);
            result
        }
    }
}

fn compile_logical_negation_ref<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    ref_name: &str,
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let symbol: SymbolAllocated = context.get_allocated_symbol(ref_name);

    match kind {
        kind if kind.is_integer_type() || kind.is_bool_type() => {
            let int: IntValue = symbol.load(context).into_int_value();
            let mut result: BasicValueEnum = llvm_builder.build_not(int, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
                    result = casted_int;
                }
            }

            result
        }
        _ => {
            let float: FloatValue = symbol.load(context).into_float_value();
            let mut result: BasicValueEnum =
                llvm_builder.build_float_neg(float, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                    result = casted_float;
                }
            }

            result
        }
    }
}

fn compile_arithmetic_negation_ref<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let symbol: SymbolAllocated = context.get_allocated_symbol(name);

    match kind {
        kind if kind.is_integer_type() => {
            let int: IntValue = symbol.load(context).into_int_value();
            let mut result: BasicValueEnum = llvm_builder.build_not(int, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
                    result = casted_int;
                }
            }

            result
        }
        _ => {
            let float: FloatValue = symbol.load(context).into_float_value();
            let mut result: BasicValueEnum =
                llvm_builder.build_float_neg(float, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                    result = casted_float;
                }
            }

            result
        }
    }
}

fn compile_logical_negation_bool<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    value: u64,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let value: IntValue = intgen::integer(llvm_context, &ThrushType::Bool, value, false);
    let mut result: BasicValueEnum = llvm_builder.build_not(value, "").unwrap().into();

    if let Some(cast_type) = cast_type {
        if let Some(casted_int) = cast::integer(context, cast_type, &ThrushType::Bool, result) {
            result = casted_int;
        }
    }

    result
}

fn compile_arithmetic_negation_int<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx ThrushType,
    value: u64,
    signed: bool,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let mut value: IntValue = intgen::integer(llvm_context, kind, value, signed);

    match signed {
        false => {
            let mut result: IntValue = llvm_builder.build_not(value, "").unwrap();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result.into()) {
                    result = casted_int.into_int_value();
                }
            }

            result.into()
        }
        true => {
            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, value.into()) {
                    value = casted_int.into_int_value();
                }
            }

            value.into()
        }
    }
}

fn compile_arithmetic_negation_float<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx ThrushType,
    value: f64,
    signed: bool,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    let mut value: FloatValue = floatgen::float(llvm_builder, llvm_context, kind, value, signed);

    match signed {
        false => {
            let mut result: FloatValue = llvm_builder.build_float_neg(value, "").unwrap();

            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, result.into()) {
                    result = casted_float.into_float_value();
                }
            }

            result.into()
        }
        true => {
            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, value.into()) {
                    value = casted_float.into_float_value();
                }
            }

            value.into()
        }
    }
}
