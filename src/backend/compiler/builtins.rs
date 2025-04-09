use {
    super::{
        super::super::{
            frontend::{lexer::Type, objects::Functions},
            logging,
        },
        Instruction,
        memory::AllocatedObject,
        objects::CompilerObjects,
        types::{Call, CompilerStructure, CompilerStructureFields},
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

pub fn include(functions: &mut Functions) {
    functions.insert(
        "sizeof!",
        (
            Type::S64,
            Vec::from([Type::T]),
            Vec::new(),
            String::new(),
            false,
        ),
    );

    functions.insert(
        "is_signed!",
        (
            Type::Bool,
            Vec::from([Type::T]),
            Vec::new(),
            String::new(),
            false,
        ),
    );
}

pub fn build_sizeof<'ctx>(
    context: &'ctx Context,
    call: Call<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let value: &Instruction = &call.2[0];

    if let Instruction::LocalRef {
        name, kind, line, ..
    } = value
    {
        let localref_type: &Type = kind.get_type();

        if localref_type.is_struct_type() {
            let localref_structure_type: &str = kind.get_type_structure_type();
            let structure: &CompilerStructure =
                compiler_objects.get_struct(localref_structure_type);
            let structure_fields: &CompilerStructureFields = &structure.1;

            let llvm_type: StructType =
                utils::build_struct_type_from_fields(context, structure_fields);

            let structure_size: IntValue = llvm_type.size_of().unwrap_or_else(|| {
                logging::log(
                    logging::LogType::Panic,
                    &format!(
                        "Builtin 'sizeof()' cannot get the size of `{}`, line {}.",
                        name, line
                    ),
                );

                unreachable!()
            });

            return structure_size.into();
        }

        let ptr: PointerValue = compiler_objects.get_allocated_object(name).ptr;

        return ptr.get_type().size_of().into();
    }

    if let Instruction::Type(kind, _) = value {
        match kind {
            kind if kind.is_integer_type() || kind.is_bool_type() => {
                return utils::type_int_to_llvm_int_type(context, kind)
                    .size_of()
                    .into();
            }
            kind if kind.is_float_type() => {
                return utils::type_float_to_llvm_float_type(context, kind)
                    .size_of()
                    .into();
            }
            kind if *kind == Type::T => {
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
            value.get_type()
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
        let localref_type: &Type = kind.get_type();

        if !localref_type.is_float_type() && !kind.is_integer_type() {
            logging::log(
                logging::LogType::Panic,
                &format!(
                    "Builtin 'is_signed()' cannot get the signedness of `{}`, line {}.",
                    name, line
                ),
            );
        }

        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        return if kind.is_integer_type() {
            let mut loaded_value: IntValue = object
                .load_from_memory(
                    builder,
                    utils::type_int_to_llvm_int_type(context, localref_type),
                )
                .into_int_value();

            if let Some(casted_float) = utils::integer_autocast(
                &Type::S64,
                localref_type,
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
        } else {
            let mut loaded_value: FloatValue = object
                .load_from_memory(
                    builder,
                    utils::type_float_to_llvm_float_type(context, localref_type),
                )
                .into_float_value();

            if let Some(casted_float) = utils::float_autocast(
                localref_type,
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
        };
    }

    context.bool_type().const_zero().into()
}
