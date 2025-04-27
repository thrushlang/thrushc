use crate::middle::{
    statement::{Function, FunctionCall},
    types::Type,
};

use super::{
    Instruction,
    builtins::{build_is_signed, build_sizeof},
    symbols::SymbolsTable,
    valuegen,
};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue},
};

pub fn build_call<'a, 'ctx>(
    function_call: FunctionCall<'ctx>,
    symbols: &SymbolsTable<'a, 'ctx>,
) -> Option<BasicValueEnum<'ctx>> {
    let context: &Context = symbols.get_llvm_context();
    let builder: &Builder = symbols.get_llvm_builder();

    let call_name: &str = function_call.0;
    let call_args: &[Instruction<'ctx>] = function_call.2;
    let call_type: &Type = function_call.1;

    if call_name == "sizeof!" {
        return Some(build_sizeof(context, function_call, symbols));
    }

    if call_name == "is_signed!" {
        return Some(build_is_signed(context, builder, function_call, symbols));
    }

    let function: Function = symbols.get_function(call_name);

    let llvm_function: FunctionValue = function.0;

    let target_function_arguments: &[Instruction] = function.1;
    let function_convention: u32 = function.2;

    let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

    call_args.iter().enumerate().for_each(|instruction| {
        let casting_target: &Type = target_function_arguments
            .get(instruction.0)
            .unwrap_or(instruction.1)
            .get_type();

        compiled_args
            .push(valuegen::generate_expression(instruction.1, casting_target, symbols).into());
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
