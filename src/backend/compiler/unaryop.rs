use {
    super::{
        super::super::frontend::lexer::TokenKind, objects::CompilerObjects, types::UnaryOp, utils,
        Instruction,
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
    unary: UnaryOp,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let (
        TokenKind::PlusPlus | TokenKind::MinusMinus,
        Instruction::RefVar {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let variable: PointerValue<'ctx> = compiler_objects.find_and_get(name).unwrap();

        if refvar_type.is_integer_type() {
            let left_num: IntValue<'ctx> = builder
                .build_load(
                    utils::datatype_integer_to_llvm_type(context, refvar_type),
                    variable,
                    "",
                )
                .unwrap()
                .into_int_value();

            let right_num: IntValue<'ctx> =
                utils::datatype_integer_to_llvm_type(context, refvar_type).const_int(1, false);

            let result: IntValue<'ctx> = if *unary.0 == TokenKind::PlusPlus {
                builder.build_int_nsw_add(left_num, right_num, "").unwrap()
            } else {
                builder.build_int_nsw_sub(left_num, right_num, "").unwrap()
            };

            builder.build_store(variable, result).unwrap();

            return result.into();
        }

        let left_num: FloatValue<'ctx> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, refvar_type),
                variable,
                "",
            )
            .unwrap()
            .into_float_value();

        let right_num: FloatValue<'ctx> =
            utils::datatype_float_to_llvm_type(context, refvar_type).const_float(1.0);

        let result: FloatValue<'ctx> = if *unary.0 == TokenKind::PlusPlus {
            builder.build_float_add(left_num, right_num, "").unwrap()
        } else {
            builder.build_float_sub(left_num, right_num, "").unwrap()
        };

        builder.build_store(variable, result).unwrap();

        return result.into();
    }

    if let (
        TokenKind::Bang,
        Instruction::RefVar {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let variable: PointerValue<'ctx> = compiler_objects.find_and_get(name).unwrap();

        if refvar_type.is_integer_type() {
            let left: IntValue<'ctx> = builder
                .build_load(
                    utils::datatype_integer_to_llvm_type(context, refvar_type),
                    variable,
                    "",
                )
                .unwrap()
                .into_int_value();

            let result: IntValue<'ctx> = builder.build_not(left, "").unwrap();

            builder.build_store(variable, result).unwrap();

            return result.into();
        }

        let left: FloatValue<'ctx> = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, refvar_type),
                variable,
                "",
            )
            .unwrap()
            .into_float_value();

        let result: FloatValue<'ctx> = builder.build_float_neg(left, "").unwrap();

        builder.build_store(variable, result).unwrap();

        return result.into();
    }

    unreachable!()
}
