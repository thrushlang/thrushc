use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::parser::stmts::stmt::ThrushStatement;
use crate::frontend::types::representations::UnaryOperation;

use super::{context::LLVMCodeGenContext, memory::SymbolAllocated, valuegen};

use super::typegen;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn unary_op<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    unary: UnaryOperation<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();
    let llvm_context: &Context = context.get_llvm_context();

    if let (
        TokenType::PlusPlus | TokenType::MinusMinus,
        _,
        ThrushStatement::Reference {
            name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if ref_type.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();

            let modifier: IntValue =
                typegen::thrush_integer_to_llvm_type(llvm_context, ref_type).const_int(1, false);

            let result: IntValue = if unary.0.is_plusplus_operator() {
                llvm_builder.build_int_nsw_add(int, modifier, "").unwrap()
            } else {
                llvm_builder.build_int_nsw_sub(int, modifier, "").unwrap()
            };

            symbol.store(context, result.into());

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();

        let modifier: FloatValue =
            typegen::type_float_to_llvm_float_type(llvm_context, ref_type).const_float(1.0);

        let result: FloatValue = if unary.0.is_plusplus_operator() {
            llvm_builder.build_float_add(float, modifier, "").unwrap()
        } else {
            llvm_builder.build_float_sub(float, modifier, "").unwrap()
        };

        symbol.store(context, result.into());

        return result.into();
    }

    if let (
        TokenType::Bang,
        _,
        ThrushStatement::Reference {
            name: ref_name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(ref_name);

        if ref_type.is_integer_type() || ref_type.is_bool_type() {
            let int: IntValue = symbol.load(context).into_int_value();
            let result: IntValue = llvm_builder.build_not(int, "").unwrap();

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();
        let result: FloatValue = llvm_builder.build_float_neg(float, "").unwrap();

        return result.into();
    }

    if let (
        TokenType::Minus,
        _,
        ThrushStatement::Reference {
            name,
            kind: ref_type,
            ..
        },
    ) = unary
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if ref_type.is_integer_type() {
            let int: IntValue = symbol.load(context).into_int_value();
            let result: IntValue = llvm_builder.build_not(int, "").unwrap();

            return result.into();
        }

        let float: FloatValue = symbol.load(context).into_float_value();
        let result: FloatValue = llvm_builder.build_float_neg(float, "").unwrap();

        return result.into();
    }

    if let (TokenType::Bang, _, ThrushStatement::Boolean { value, .. }) = unary {
        let value: IntValue = valuegen::integer(llvm_context, &ThrushType::Bool, *value, false);
        let result: IntValue = llvm_builder.build_not(value, "").unwrap();

        return result.into();
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
        let value: IntValue = valuegen::integer(llvm_context, kind, *value, *signed);

        if !signed {
            let result: IntValue = llvm_builder.build_not(value, "").unwrap();
            return result.into();
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
        let value: FloatValue = valuegen::float(llvm_builder, llvm_context, kind, *value, *signed);

        if !signed {
            let result: FloatValue = llvm_builder.build_float_neg(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    unreachable!()
}
