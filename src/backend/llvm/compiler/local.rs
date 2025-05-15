use crate::middle::types::{backend::llvm::types::LLVMLocal, frontend::lexer::types::ThrushType};

use super::{
    Instruction,
    context::{LLVMCodeGenContext, LLVMCodeGenContextPosition},
    memory::SymbolAllocated,
    valuegen,
};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};

pub fn build<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_type: &ThrushType = local.1;

    context.alloc_local(local.0, local.1);
    context.set_position(LLVMCodeGenContextPosition::Local);

    if local_type.is_ptr_type() {
        build_local_ptr(local, context);
    }

    if local_type.is_str_type() {
        build_local_str(local, context);
    }

    if local_type.is_struct_type() {
        build_local_structure(local, context);
    }

    if local_type.is_integer_type() {
        build_local_integer(local, context);
    }

    if local_type.is_float_type() {
        build_local_float(local, context);
    }

    if local_type.is_bool_type() {
        build_local_boolean(local, context);
    }

    if local_type.is_mut_type() {
        build_local_mut(local, context);
    }

    context.set_position_irrelevant();
}

fn build_local_mut<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::generate_expression(local_value, local.1, context);

    symbol.store(context, expression);
}

fn build_local_ptr<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, &ThrushType::Ptr(None), context);

    symbol.store(context, expression);
}

fn build_local_structure<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    if let Instruction::Constructor { arguments, .. } = local_value {
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_context: &Context = context.get_llvm_context();

        arguments.1.iter().for_each(|argument| {
            let argument_instruction: &Instruction = &argument.1;
            let argument_type: &ThrushType = &argument.2;
            let index: u32 = argument.3;

            let compiled_field: BasicValueEnum =
                valuegen::generate_expression(argument_instruction, argument_type, context);

            let get_field: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, index);

            llvm_builder.build_store(get_field, compiled_field).unwrap();
        });

        return;
    }

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_str<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::generate_expression(local_value, local.1, context);

    symbol.store(context, expression);
}

fn build_local_integer<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_float<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_boolean<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, context);

    symbol.store(context, expression);
}
