use {
    super::{
        super::super::frontend::lexer::DataTypes, generation, objects::CompilerObjects,
        types::Call, utils, Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
        AddressSpace,
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
    let call_type: &DataTypes = call.1;

    if call_name == "sizeof" {
        return build_sizeof(context, call, compiler_objects);
    }

    let function: (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>]) =
        compiler_objects.get_function(call_name).unwrap();

    let called_function_arguments: &[Instruction] = function.1;
    let called_function: FunctionValue = function.0;

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    call_args.iter().enumerate().for_each(|instruction| {
        let casting_target: Option<DataTypes> = called_function_arguments
            .get(instruction.0)
            .map(|arg| arg.get_data_type());

        compiled_args.push(
            generation::build_basic_value_enum(
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

    if let Instruction::RefVar { name, .. } = var_value {
        let ptr: PointerValue<'ctx> = compiler_objects.get_local(name).unwrap();
        return Some(ptr.get_type().size_of().into());
    }

    if let Instruction::DataTypes(data_type) = var_value {
        match data_type {
            data_type if data_type.is_integer_type() || *data_type == DataTypes::Bool => {
                return Some(
                    utils::datatype_integer_to_llvm_type(context, data_type)
                        .size_of()
                        .into(),
                );
            }
            data_type if data_type.is_float_type() => {
                return Some(
                    utils::datatype_float_to_llvm_type(context, data_type)
                        .size_of()
                        .into(),
                );
            }
            data_type if *data_type == DataTypes::Ptr => {
                return Some(context.ptr_type(AddressSpace::default()).size_of().into());
            }

            _ => unreachable!(),
        }
    }

    None
}
