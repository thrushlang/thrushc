use {
    super::{
        super::super::frontend::lexer::DataTypes, codegen, objects::CompilerObjects, types::Call,
        Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicMetadataValueEnum, BasicValueEnum},
    },
};

pub fn build_call<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    call: Call<'ctx>,
    objects: &CompilerObjects<'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let call_name: &str = call.0;
    let call_args: &[Instruction<'ctx>] = call.2;
    let call_type: &DataTypes = call.1;

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    call_args.iter().for_each(|arg| {
        compiled_args.push(
            codegen::build_basic_value_enum(
                module,
                builder,
                context,
                arg,
                &[],
                arg.is_var(),
                objects,
            )
            .into(),
        );
    });

    if *call_type != DataTypes::Void {
        Some(
            builder
                .build_call(
                    objects.find_and_get_function(call_name).unwrap(),
                    &compiled_args,
                    "",
                )
                .unwrap()
                .try_as_basic_value()
                .unwrap_left(),
        )
    } else {
        builder
            .build_call(
                objects.find_and_get_function(call_name).unwrap(),
                &compiled_args,
                "",
            )
            .unwrap();

        None
    }
}
