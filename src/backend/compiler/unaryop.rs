use {
    super::{
        super::super::frontend::lexer::{TokenKind, Type},
        Instruction,
        objects::CompilerObjects,
        types::UnaryOp,
        utils,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    },
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
        let ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        if refvar_type.is_integer_type() {
            let left_num: IntValue = builder
                .build_load(
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                    ptr,
                    "",
                )
                .unwrap()
                .into_int_value();

            let right_num: IntValue =
                utils::type_int_to_llvm_int_type(context, refvar_type).const_int(1, false);

            let result: IntValue = if unary.0.is_plusplus_operator() {
                builder.build_int_nsw_add(left_num, right_num, "").unwrap()
            } else {
                builder.build_int_nsw_sub(left_num, right_num, "").unwrap()
            };

            builder.build_store(ptr, result).unwrap();

            return result.into();
        }

        let left_num: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, refvar_type),
                ptr,
                "",
            )
            .unwrap()
            .into_float_value();

        let right_num: FloatValue =
            utils::type_float_to_llvm_float_type(context, refvar_type).const_float(1.0);

        let result: FloatValue = if unary.0.is_plusplus_operator() {
            builder.build_float_add(left_num, right_num, "").unwrap()
        } else {
            builder.build_float_sub(left_num, right_num, "").unwrap()
        };

        builder.build_store(ptr, result).unwrap();

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
        let ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        if refvar_type.is_integer_type() || refvar_type.is_bool_type() {
            let left: IntValue = builder
                .build_load(
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                    ptr,
                    "",
                )
                .unwrap()
                .into_int_value();

            let result: IntValue = builder.build_not(left, "").unwrap();

            return result.into();
        }

        let left: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, refvar_type),
                ptr,
                "",
            )
            .unwrap()
            .into_float_value();

        let result: FloatValue = builder.build_float_neg(left, "").unwrap();
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
        let ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        if refvar_type.is_integer_type() {
            let left: IntValue = builder
                .build_load(
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                    ptr,
                    "",
                )
                .unwrap()
                .into_int_value();

            let result: IntValue = builder.build_not(left, "").unwrap();

            return result.into();
        }

        let left: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, refvar_type),
                ptr,
                "",
            )
            .unwrap()
            .into_float_value();

        let result: FloatValue = builder.build_float_neg(left, "").unwrap();

        return result.into();
    }

    if let (TokenKind::Bang, Instruction::Boolean(bool), _) = unary {
        let value: IntValue = utils::build_const_integer(context, &Type::Bool, *bool as u64, false);
        let result: IntValue = builder.build_not(value, "").unwrap();

        return result.into();
    }

    if let (TokenKind::Minus, Instruction::Integer(integer_type, num, is_signed), _) = unary {
        let value: IntValue =
            utils::build_const_integer(context, integer_type, *num as u64, *is_signed);

        if !is_signed {
            let result: IntValue = builder.build_not(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    if let (TokenKind::Minus, Instruction::Float(float_type, num, is_signed), _) = unary {
        let value: FloatValue =
            utils::build_const_float(builder, context, float_type, *num, *is_signed);

        if !is_signed {
            let result: FloatValue = builder.build_float_neg(value, "").unwrap();
            return result.into();
        }

        return value.into();
    }

    unreachable!()
}
