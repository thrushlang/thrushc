use {
    super::{
        super::super::{frontend::lexer::Type, logging},
        Instruction,
        objects::CompilerObjects,
        types::Call,
        utils,
    },
    inkwell::{
        AddressSpace, FloatPredicate,
        builder::Builder,
        context::Context,
        types::StructType,
        values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    },
};

pub fn build_sizeof<'ctx>(
    context: &'ctx Context,
    call: Call<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let value: &Instruction = &call.2[0];

    if let Instruction::LocalRef {
        name,
        struct_type,
        line,
        ..
    } = value
    {
        if let Some(structure_fields) = compiler_objects.get_struct(struct_type) {
            let structure_type: StructType =
                utils::build_struct_type_from_fields(context, structure_fields);

            let structure_size_of: IntValue = structure_type.size_of().unwrap_or_else(|| {
                logging::log(
                    logging::LogType::Panic,
                    &format!(
                        "Builtin 'sizeof()' cannot get the size of `{}`, line {}.",
                        name, line
                    ),
                );

                unreachable!()
            });

            return structure_size_of.into();
        }

        let ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        return ptr.get_type().size_of().into();
    }

    if let Instruction::Type(type_) = value {
        match type_ {
            type_ if type_.is_integer_type() || type_.is_bool_type() => {
                return utils::type_int_to_llvm_int_type(context, type_)
                    .size_of()
                    .into();
            }
            type_ if type_.is_float_type() => {
                return utils::type_float_to_llvm_float_type(context, type_)
                    .size_of()
                    .into();
            }
            type_ if *type_ == Type::Ptr => {
                return context.ptr_type(AddressSpace::default()).size_of().into();
            }

            what => {
                logging::log(
                    logging::LogType::Panic,
                    &format!("Builtin 'sizeof()' cannot get the size of '{}' type.", what),
                );

                unreachable!()
            }
        }
    }

    logging::log(
        logging::LogType::Panic,
        &format!(
            "Builtin 'sizeof()' cannot get the size of '{}' type.",
            value.get_data_type()
        ),
    );

    unreachable!()
}

pub fn build_is_signed<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    call: Call<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let value: &Instruction = &call.2[0];

    if let Instruction::LocalRef {
        name, kind, line, ..
    } = value
    {
        if !kind.is_float_type() && !kind.is_integer_type() {
            logging::log(
                logging::LogType::Panic,
                &format!(
                    "Builtin 'is_signed()' cannot get the signedness of `{}`, line {}.",
                    name, line
                ),
            );
        }

        let ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        return if kind.is_float_type() {
            let mut loaded_value: FloatValue = builder
                .build_load(utils::type_float_to_llvm_float_type(context, kind), ptr, "")
                .unwrap()
                .into_float_value();

            if let Some(casted_float) = utils::float_autocast(
                kind,
                &Type::F64,
                None,
                loaded_value.into(),
                builder,
                context,
            ) {
                loaded_value = casted_float.into_float_value();
            }

            builder
                .build_float_compare(
                    FloatPredicate::OLT,
                    loaded_value,
                    context.f64_type().const_float(0.0),
                    "",
                )
                .unwrap()
                .into()
        } else {
            let mut loaded_value: IntValue = builder
                .build_load(utils::type_int_to_llvm_int_type(context, kind), ptr, "")
                .unwrap()
                .into_int_value();

            if let Some(casted_float) = utils::integer_autocast(
                kind,
                &Type::I64,
                None,
                loaded_value.into(),
                builder,
                context,
            ) {
                loaded_value = casted_float.into_int_value();
            }

            builder
                .build_int_compare(
                    inkwell::IntPredicate::SLT,
                    loaded_value,
                    context.i64_type().const_int(0, false),
                    "",
                )
                .unwrap()
                .into()
        };
    }

    context.bool_type().const_zero().into()
}
