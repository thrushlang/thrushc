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
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if let (
        TokenType::PlusPlus | TokenType::MinusMinus,
        _,
        ThrushStatement::Reference { name, kind, .. },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);
        let operator: &TokenType = unary.0;

        if kind.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue =
                typegen::thrush_integer_to_llvm_type(llvm_context, kind).const_int(1, false);

            let mut result: BasicValueEnum = if operator.is_plusplus_operator() {
                if let Ok(result) = llvm_builder.build_int_nsw_add(int, modifier, "") {
                    result.into()
                } else {
                    logging::log(
                        logging::LoggingType::Bug,
                        "Failed to compile an incrementer.",
                    );

                    unreachable!()
                }
            } else if let Ok(result) = llvm_builder.build_int_nsw_sub(int, modifier, "") {
                result.into()
            } else {
                logging::log(
                    logging::LoggingType::Bug,
                    "Failed to compile an decrementer.",
                );

                unreachable!()
            };

            symbol.store(context, result);

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
                    result = casted_int;
                }
            }

            return result;
        }

        let float: FloatValue = symbol.load(context).into_float_value();

        let modifier: FloatValue =
            typegen::type_float_to_llvm_float_type(llvm_context, kind).const_float(1.0);

        let mut result: BasicValueEnum = if operator.is_plusplus_operator() {
            llvm_builder
                .build_float_add(float, modifier, "")
                .unwrap()
                .into()
        } else {
            llvm_builder
                .build_float_sub(float, modifier, "")
                .unwrap()
                .into()
        };

        if let Some(cast_type) = cast_type {
            if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                result = casted_float;
            }
        }

        symbol.store(context, result);

        return result;
    }

    if let (
        TokenType::Bang,
        _,
        ThrushStatement::Reference {
            name: ref_name,
            kind,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(ref_name);

        if kind.is_integer_type() || kind.is_bool_type() {
            let int: IntValue = symbol.load(context).into_int_value();

            let mut result: BasicValueEnum = llvm_builder.build_not(int, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
                    result = casted_int;
                }
            }

            return result;
        }

        let float: FloatValue = symbol.load(context).into_float_value();

        let mut result: BasicValueEnum = llvm_builder.build_float_neg(float, "").unwrap().into();

        if let Some(cast_type) = cast_type {
            if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                result = casted_float;
            }
        }

        return result;
    }

    if let (TokenType::Minus, _, ThrushStatement::Reference { name, kind, .. }) = unary {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if kind.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();

            let mut result: BasicValueEnum = llvm_builder.build_not(int, "").unwrap().into();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result) {
                    result = casted_int;
                }
            }

            return result;
        }

        let float: FloatValue = symbol.load(context).into_float_value();

        let mut result: BasicValueEnum = llvm_builder.build_float_neg(float, "").unwrap().into();

        if let Some(cast_type) = cast_type {
            if let Some(casted_float) = cast::float(context, cast_type, kind, result) {
                result = casted_float;
            }
        }

        return result;
    }

    if let (TokenType::Bang, _, ThrushStatement::Boolean { value, .. }) = unary {
        let value: IntValue = intgen::integer(llvm_context, &ThrushType::Bool, *value, false);

        let mut result: BasicValueEnum = llvm_builder.build_not(value, "").unwrap().into();

        if let Some(cast_type) = cast_type {
            if let Some(casted_int) = cast::integer(context, cast_type, &ThrushType::Bool, result) {
                result = casted_int;
            }
        }

        return result;
    }

    if let (
        TokenType::Minus,
        _,
        ThrushStatement::Integer {
            kind,
            value,
            signed,
            ..
        },
    ) = unary
    {
        let mut value: IntValue = intgen::integer(llvm_context, kind, *value, *signed);

        if !signed {
            let mut result: IntValue = llvm_builder.build_not(value, "").unwrap();

            if let Some(cast_type) = cast_type {
                if let Some(casted_int) = cast::integer(context, cast_type, kind, result.into()) {
                    result = casted_int.into_int_value();
                }
            }

            return result.into();
        }

        if let Some(cast_type) = cast_type {
            if let Some(casted_int) = cast::integer(context, cast_type, kind, value.into()) {
                value = casted_int.into_int_value();
            }
        }

        return value.into();
    }

    if let (
        TokenType::Minus,
        _,
        ThrushStatement::Float {
            kind,
            value,
            signed,
            ..
        },
    ) = unary
    {
        let mut value: FloatValue =
            floatgen::float(llvm_builder, llvm_context, kind, *value, *signed);

        if !signed {
            let mut result: FloatValue = llvm_builder.build_float_neg(value, "").unwrap();

            if let Some(cast_type) = cast_type {
                if let Some(casted_float) = cast::float(context, cast_type, kind, result.into()) {
                    result = casted_float.into_float_value();
                }
            }

            return result.into();
        }

        if let Some(cast_type) = cast_type {
            if let Some(casted_float) = cast::float(context, cast_type, kind, value.into()) {
                value = casted_float.into_float_value();
            }
        }

        return value.into();
    }

    unreachable!()
}
