use {
    super::{
        super::super::frontend::lexer::TokenKind, objects::CompilerObjects, utils, Instruction,
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
    instr: &Instruction<'ctx>,
    objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = instr
    {
        if let (TokenKind::PlusPlus, Instruction::RefVar { name, kind, .. }, _) =
            (op, &**value, kind)
        {
            let variable: PointerValue<'ctx> = objects.find_and_get(name).unwrap();

            if kind.is_integer() {
                let left_num: IntValue<'ctx> = builder
                    .build_load(
                        utils::datatype_integer_to_llvm_type(context, kind),
                        variable,
                        "",
                    )
                    .unwrap()
                    .into_int_value();

                let right_num: IntValue<'ctx> =
                    utils::datatype_integer_to_llvm_type(context, kind).const_int(1, false);

                let result: IntValue<'ctx> =
                    builder.build_int_nsw_add(left_num, right_num, "").unwrap();

                builder.build_store(variable, result).unwrap();

                return result.into();
            }

            let left_num: FloatValue<'ctx> = builder
                .build_load(
                    utils::datatype_float_to_llvm_type(context, kind),
                    variable,
                    "",
                )
                .unwrap()
                .into_float_value();

            let right_num: FloatValue<'ctx> =
                utils::datatype_float_to_llvm_type(context, kind).const_float(1.0);

            let result: FloatValue<'ctx> =
                builder.build_float_add(left_num, right_num, "").unwrap();

            builder.build_store(variable, result).unwrap();

            return result.into();
        }
    }

    unreachable!()
}
