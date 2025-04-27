use crate::middle::{statement::Local, types::Type};

use super::{Instruction, memory::SymbolAllocated, symbols::SymbolsTable, valuegen};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};

pub fn build<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let local_type: &Type = local.1;

    symbols.alloc_local(local.0, local.1);

    if local_type.is_ptr_type() {
        build_local_ptr(local, symbols);
        return;
    }

    if local_type.is_str_type() {
        build_local_str(local, symbols);
        return;
    }

    if local_type.is_struct_type() {
        build_local_structure(local, symbols);
        return;
    }

    if local_type.is_integer_type() {
        build_local_integer(local, symbols);
        return;
    }

    if local_type.is_float_type() {
        build_local_float(local, symbols);
        return;
    }

    if local_type.is_bool_type() {
        build_local_boolean(local, symbols);
        return;
    }

    unreachable!()
}

pub fn build_local_mutation<'ctx>(symbols: &mut SymbolsTable<'_, 'ctx>, local: Local<'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local_name);

    if let Instruction::LocalMut { value, .. } = local_value {
        let expression: BasicValueEnum = valuegen::generate_expression(value, local_type, symbols);

        symbol.store(builder, expression);

        return;
    }

    if local_type.is_integer_type() {
        build_local_integer(local, symbols);
        return;
    }

    if local_type.is_float_type() {
        build_local_float(local, symbols);
        return;
    }

    if local_type.is_bool_type() {
        build_local_boolean(local, symbols);
        return;
    }

    if local_type.is_ptr_type() {
        build_local_ptr(local, symbols);
        return;
    }

    todo!()
}

fn build_local_ptr<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local.0);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, &Type::Ptr(None), symbols);

    symbol.store(builder, expression);
}

fn build_local_structure<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local.0);

    if let Instruction::InitStruct { arguments, .. } = local_value {
        let context: &Context = symbols.get_llvm_context();

        arguments.iter().for_each(|argument| {
            let argument_instruction: &Instruction = &argument.1;
            let argument_type: &Type = &argument.2;
            let index: u32 = argument.3;

            let compiled_field: BasicValueEnum =
                valuegen::generate_expression(argument_instruction, argument_type, symbols);

            let get_field: PointerValue = symbol.gep_struct(context, builder, index);

            builder.build_store(get_field, compiled_field).unwrap();
        });

        return;
    }

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, symbols);

    symbol.store(builder, expression);
}

fn build_local_str<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::generate_expression(local_value, local.1, symbols);

    symbol.store(builder, expression);
}

fn build_local_integer<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, symbols);

    symbol.store(builder, expression);
}

fn build_local_float<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, symbols);

    symbol.store(builder, expression);
}

fn build_local_boolean<'ctx>(local: Local<'ctx>, symbols: &mut SymbolsTable<'_, 'ctx>) {
    let builder: &Builder = symbols.get_llvm_builder();

    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(local_value, local_type, symbols);

    symbol.store(builder, expression);
}
