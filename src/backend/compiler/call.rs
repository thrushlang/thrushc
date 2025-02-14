use {
    super::{
        super::super::frontend::lexer::DataTypes, codegen, impls::BasicValueEnumExt,
        objects::CompilerObjects, types::Call, utils, Instruction,
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
    compiler_objects: &CompilerObjects<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let call_name: &str = call.0;
    let call_args: &[Instruction<'ctx>] = call.2;
    let call_type: &DataTypes = call.1;

    if call_name == "sizeof" {
        return build_sizeof(context, call, compiler_objects);
    }

    let called_function: FunctionValue<'ctx> =
        compiler_objects.find_and_get_function(call_name).unwrap();

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    let mut index: usize = 0;

    call_args.iter().for_each(|instr| {
        let arg: Option<DataTypes> = called_function
            .get_nth_param(index as u32)
            .map(|arg| arg.get_data_type(context));

        compiled_args.push(
            codegen::build_basic_value_enum(module, builder, context, instr, arg, compiler_objects)
                .into(),
        );

        index += 1;
    });

    if *call_type != DataTypes::Void {
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
        let ptr: PointerValue<'ctx> = compiler_objects.find_and_get(name).unwrap();

        return Some(ptr.get_type().size_of().into());
    }

    if let Instruction::DataTypes(data_type) = var_value {
        match data_type {
            data_type if data_type.is_integer() || *data_type == DataTypes::Bool => {
                return Some(
                    utils::datatype_integer_to_llvm_type(context, data_type)
                        .size_of()
                        .into(),
                );
            }

            data_type if data_type.is_float() => {
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
