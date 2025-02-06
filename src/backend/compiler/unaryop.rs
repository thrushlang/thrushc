use {
    super::{
        super::super::frontend::lexer::{DataTypes, TokenKind},
        objects::CompilerObjects,
        utils, Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue, StructValue},
    },
};

pub fn compile_unary_op<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instr: &Instruction<'ctx>,
    objects: &CompilerObjects<'ctx>,
    function: FunctionValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::Unary {
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

                let result: StructValue<'_> = match kind {
                    DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 => builder
                        .build_call(
                            module
                                .get_function(&format!(
                                    "llvm.sadd.with.overflow.{}",
                                    kind.as_llvm_identifier()
                                ))
                                .unwrap(),
                            &[left_num.into(), right_num.into()],
                            "",
                        )
                        .unwrap()
                        .try_as_basic_value()
                        .unwrap_left(),

                    _ => unreachable!(),
                }
                .into_struct_value();

                let result = utils::build_possible_overflow(
                    module,
                    context,
                    builder,
                    result,
                    instr.get_unary_data_for_overflow(),
                    function,
                    None,
                );

                builder.build_store(variable, result).unwrap();

                return result;
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
