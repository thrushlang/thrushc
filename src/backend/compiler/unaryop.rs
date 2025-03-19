use {
    super::{
        super::super::frontend::lexer::TokenKind, Instruction, objects::CompilerObjects,
        types::UnaryOp, utils,
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
        Instruction::LocalRef {
            name,
            kind: refvar_type,
            ..
        },
        _,
    ) = unary
    {
        let variable: PointerValue = compiler_objects.get_local(name).unwrap();

        if refvar_type.is_integer_type() {
            let left_num: IntValue<'ctx> = builder
                .build_load(
                    utils::type_int_to_llvm_int_type(context, refvar_type),
                    variable,
                    "",
                )
                .unwrap()
                .into_int_value();

            let right_num: IntValue =
                utils::type_int_to_llvm_int_type(context, refvar_type).const_int(1, false);

            let result: IntValue = if *unary.0 == TokenKind::PlusPlus {
                builder.build_int_nsw_add(left_num, right_num, "").unwrap()
            } else {
                builder.build_int_nsw_sub(left_num, right_num, "").unwrap()
            };

            builder.build_store(variable, result).unwrap();

            return result.into();
        }

        let left_num: FloatValue = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, refvar_type),
                variable,
                "",
            )
            .unwrap()
            .into_float_value();

        let right_num: FloatValue =
            utils::type_float_to_llvm_float_type(context, refvar_type).const_float(1.0);

        let result: FloatValue = if *unary.0 == TokenKind::PlusPlus {
            builder.build_float_add(left_num, right_num, "").unwrap()
        } else {
            builder.build_float_sub(left_num, right_num, "").unwrap()
        };

        builder.build_store(variable, result).unwrap();

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
            builder.build_store(ptr, result).unwrap();

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
        builder.build_store(ptr, result).unwrap();

        return result.into();
    }

    unreachable!()
}
