use {
    super::{
        super::super::{frontend::lexer::Type, logging},
        Instruction, generation,
        objects::CompilerObjects,
        types::Call,
        utils,
    },
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        types::StructType,
        values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
    },
};

pub fn build_call<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    call: Call<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let call_name: &str = call.0;
    let call_args: &[Instruction<'ctx>] = call.2;
    let call_type: &Type = call.1;

    if call_name == "sizeof" {
        return build_sizeof(context, call, compiler_objects);
    }

    let function: (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>]) =
        compiler_objects.get_function(call_name).unwrap();

    let called_function_arguments: &[Instruction] = function.1;
    let called_function: FunctionValue = function.0;

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    call_args.iter().enumerate().for_each(|instruction| {
        let casting_target: Option<Type> = called_function_arguments
            .get(instruction.0)
            .map(|arg| arg.get_data_type());

        compiled_args.push(
            generation::build_expression(
                module,
                builder,
                context,
                instruction.1,
                casting_target,
                compiler_objects,
            )
            .into(),
        );
    });

    if !call_type.is_void_type() {
        Some(
            builder
                .build_call(called_function, &compiled_args, "")
                .unwrap()
                .try_as_basic_value()
                .unwrap_left(),
        )
    } else {
        builder
            .build_call(called_function, &compiled_args, "")
            .unwrap();

        None
    }
}

fn build_sizeof<'ctx>(
    context: &'ctx Context,
    call: Call<'ctx>,
    compiler_objects: &CompilerObjects<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let var_value: &Instruction<'ctx> = &call.2[0];

    if let Instruction::LocalRef {
        name,
        struct_type,
        line,
        ..
    } = var_value
    {
        if let Some(structure_fields) = compiler_objects.get_struct(struct_type) {
            let structure_type: StructType =
                utils::build_struct_type_from_fields(context, structure_fields);

            let structure_size_of: IntValue = structure_type.size_of().unwrap_or_else(|| {
                logging::log(
                    logging::LogType::Panic,
                    &format!(
                        "Built-in `sizeof()` cannot get the size of `{}`, line {}.",
                        name, line
                    ),
                );

                unreachable!()
            });

            return Some(structure_size_of.into());
        }

        let ptr: PointerValue<'ctx> = compiler_objects.get_local(name).unwrap();
        return Some(ptr.get_type().size_of().into());
    }

    if let Instruction::Type(type_) = var_value {
        match type_ {
            type_ if type_.is_integer_type() || type_.is_bool_type() => {
                return Some(
                    utils::datatype_integer_to_llvm_type(context, type_)
                        .size_of()
                        .into(),
                );
            }
            type_ if type_.is_float_type() => {
                return Some(
                    utils::datatype_float_to_llvm_type(context, type_)
                        .size_of()
                        .into(),
                );
            }
            type_ if *type_ == Type::Ptr => {
                return Some(context.ptr_type(AddressSpace::default()).size_of().into());
            }

            _ => unreachable!(),
        }
    }

    None
}
