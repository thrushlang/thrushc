use super::super::super::frontend::lexer::{TokenKind, Type};

use super::{
    Instruction, memory::AllocatedObject, objects::CompilerObjects, types::UnaryOp, utils,
};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub fn compile_unary_op<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    unary: UnaryOp<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        TokenKind::PlusPlus | TokenKind::MinusMinus,
        Instruction::LocalRef {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let refvar_type: &Type = refvar_type.get_basic_type();
        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        if refvar_type.is_integer_type() {
            let int: IntValue = object
                .load_from_memory(
                    builder,
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                )
                .into_int_value();

            let modifier: IntValue =
                utils::type_int_to_llvm_int_type(context, refvar_type).const_int(1, false);

            let result: IntValue = if unary.0.is_plusplus_operator() {
                builder.build_int_nsw_add(int, modifier, "").unwrap()
            } else {
                builder.build_int_nsw_sub(int, modifier, "").unwrap()
            };

            object.build_store(builder, result);

            return result.into();
        }

        let float: FloatValue = object
            .load_from_memory(
                builder,
                utils::type_float_to_llvm_float_type(context, refvar_type),
            )
            .into_float_value();

        let modifier: FloatValue =
            utils::type_float_to_llvm_float_type(context, refvar_type).const_float(1.0);

        let result: FloatValue = if unary.0.is_plusplus_operator() {
            builder.build_float_add(float, modifier, "").unwrap()
        } else {
            builder.build_float_sub(float, modifier, "").unwrap()
        };

        object.build_store(builder, result);

        return result.into();
    }

    if let (
        TokenKind::Bang,
        Instruction::LocalRef {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let refvar_type: &Type = refvar_type.get_basic_type();

        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        if refvar_type.is_integer_type() || refvar_type.is_bool_type() {
            let int: IntValue = object
                .load_from_memory(
                    builder,
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                )
                .into_int_value();

            let result: IntValue = builder.build_not(int, "").unwrap();
            return result.into();
        }

        let float: FloatValue = object
            .load_from_memory(
                builder,
                utils::type_float_to_llvm_float_type(context, refvar_type),
            )
            .into_float_value();

        let result: FloatValue = builder.build_float_neg(float, "").unwrap();
        return result.into();
    }

    if let (
        TokenKind::Minus,
        Instruction::LocalRef {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let refvar_type: &Type = refvar_type.get_basic_type();

        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        if refvar_type.is_integer_type() {
            let int: IntValue = object
                .load_from_memory(
                    builder,
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                )
                .into_int_value();

            let result: IntValue = builder.build_not(int, "").unwrap();
            return result.into();
        }

        let float: FloatValue = object
            .load_from_memory(
                builder,
                utils::type_float_to_llvm_float_type(context, refvar_type),
            )
            .into_float_value();

        let result: FloatValue = builder.build_float_neg(float, "").unwrap();
        return result.into();
    }

    if let (TokenKind::Bang, Instruction::Boolean(bool), _) = unary {
        let value: IntValue = utils::build_const_integer(context, &Type::Bool, *bool as u64, false);
        let result: IntValue = builder.build_not(value, "").unwrap();

        return result.into();
    }

    if let (TokenKind::Minus, Instruction::Integer(kind, num, is_signed), _) = unary {
        let integer_type: &Type = kind.get_basic_type();

        let value: IntValue =
            utils::build_const_integer(context, integer_type, *num as u64, *is_signed);

        if !is_signed {
            let result: IntValue = builder.build_not(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    if let (TokenKind::Minus, Instruction::Float(kind, num, is_signed), _) = unary {
        let float_type: &Type = kind.get_basic_type();

        let value: FloatValue =
            utils::build_const_float(builder, context, float_type, *num, *is_signed);

        if !is_signed {
            let result: FloatValue = builder.build_float_neg(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    if let (TokenKind::Minus, Instruction::EnumField { kind, value, .. }, _) = unary {
        return compile_unary_op(builder, context, (unary.0, value, kind), compiler_objects);
    }

    unreachable!()
}
