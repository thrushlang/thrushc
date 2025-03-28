use {
    super::{
        super::super::frontend::lexer::Type,
        Instruction,
        builtins::{build_is_signed, build_sizeof},
        generation,
        objects::CompilerObjects,
        types::{Call, CompilerFunction},
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue},
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

    if call_name == "sizeof!" {
        return Some(build_sizeof(context, call, compiler_objects));
    }

    if call_name == "is_signed!" {
        return Some(build_is_signed(context, builder, call, compiler_objects));
    }

    let function: CompilerFunction = compiler_objects.get_function(call_name);

    let llvm_function: FunctionValue = function.0;

    let target_function_arguments: &[Instruction] = function.1;
    let function_convention: u32 = function.2;

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    call_args.iter().enumerate().for_each(|instruction| {
        let casting_target: Type = *target_function_arguments
            .get(instruction.0)
            .unwrap_or(instruction.1)
            .get_type();

        compiled_args.push(
            generation::build_expression(
                module,
                builder,
                context,
                instruction.1,
                &casting_target,
                compiler_objects,
            )
            .into(),
        );
    });

    let call: CallSiteValue = builder
        .build_call(llvm_function, &compiled_args, "")
        .unwrap();

    call.set_call_convention(function_convention);

    if !call_type.is_void_type() {
        return Some(call.try_as_basic_value().unwrap_left());
    }

    None
}
