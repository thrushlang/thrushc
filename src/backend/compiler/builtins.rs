use super::super::super::{
    frontend::{lexer::Type, objects::Functions},
    logging::{self, LoggingType},
};

use super::{
    Instruction, memory::AllocatedObject, objects::CompilerObjects, typegen, types::FunctionCall,
    utils,
};

use inkwell::{
    FloatPredicate,
    builder::Builder,
    context::Context,
    types::StructType,
    values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
};

pub fn include(functions: &mut Functions) {
    functions.insert("sizeof!", (Type::S64, Vec::from([Type::Ptr(None)]), false));
    functions.insert(
        "is_signed!",
        (Type::Bool, Vec::from([Type::Ptr(None)]), false),
    );
}

pub fn build_sizeof<'ctx>(
    context: &'ctx Context,
    call: FunctionCall<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let value: &Instruction = &call.2[0];

    if let Instruction::LocalRef {
        name,
        kind: ref_type,
        line,
        ..
    }
    | Instruction::ConstRef {
        name,
        kind: ref_type,
        line,
        ..
    } = value
    {
        if ref_type.is_struct_type() {
            let llvm_type: StructType =
                typegen::generate_type(context, ref_type).into_struct_type();

            let structure_size: IntValue = llvm_type.size_of().unwrap_or_else(|| {
                logging::log(
                    LoggingType::Panic,
                    &format!(
                        "Built-in 'sizeof()' cannot get the size of local reference '{}' at line '{}'.",
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

    /*if let Instruction::ComplexType(kind, _, _) = value {
        match kind {
            kind if kind.is_integer_type() || kind.is_bool_type() => {
                return typegen::type_int_to_llvm_int_type(context, kind)
                    .size_of()
                    .into();
            }

            kind if kind.is_float_type() => {
                return typegen::type_float_to_llvm_float_type(context, kind)
                    .size_of()
                    .into();
            }

            kind if kind.is_ptr_type() => {
                return context.ptr_type(AddressSpace::default()).size_of().into();
            }

            what => {
                logging::log(
                    LoggingType::Panic,
                    &format!(
                        "Built-in 'sizeof()' cannot get the size of '{}' type.",
                        what
                    ),
                );

                unreachable!()
            }
        }
    } */

    logging::log(
        LoggingType::Panic,
        &format!(
            "Built-in 'sizeof()' cannot get the size of '{}' type.",
            value.get_type()
        ),
    );

    unreachable!()
}

pub fn build_is_signed<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    call: FunctionCall<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let value: &Instruction = &call.2[0];

    if let Instruction::LocalRef {
        name,
        kind: ref_type,
        line,
        ..
    }
    | Instruction::ConstRef {
        name,
        kind: ref_type,
        line,
        ..
    } = value
    {
        if !ref_type.is_float_type() && !ref_type.is_integer_type() {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Built-in 'is_signed()' cannot get the signedness of `{}`, line {}.",
                    name, line
                ),
            );
        }

        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        return if ref_type.is_integer_type() {
            let mut loaded_value: IntValue = object
                .load_from_memory(builder, typegen::generate_type(context, ref_type))
                .into_int_value();

            if let Some(casted_float) = utils::integer_autocast(
                &Type::S64,
                ref_type,
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
                .load_from_memory(builder, typegen::generate_type(context, ref_type))
                .into_float_value();

            if let Some(casted_float) = utils::float_autocast(
                &Type::F64,
                ref_type,
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
