use crate::middle::{statement::Local, types::Type};

use super::{Instruction, call, memory::AllocatedSymbol, symbols::SymbolsTable, typegen, valuegen};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, PointerValue},
};

pub fn build<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_type: &Type = local.1;

    if local_type.is_ptr_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_ptr(module, builder, context, local, symbols);

        return;
    }

    if local_type.is_str_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_str(module, builder, context, local, symbols);

        return;
    }

    if local_type.is_struct_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_structure(module, builder, context, local, symbols);

        return;
    }

    if local_type.is_integer_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_integer(module, builder, context, local, symbols);

        return;
    }

    if local_type.is_float_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_float(module, builder, context, local, symbols);

        return;
    }

    if local_type.is_bool_type() {
        symbols.alloc(local.0, local.1, &local.3);
        build_local_boolean(module, builder, context, local, symbols);
    }

    unreachable!()
}

pub fn build_local_mutation<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    symbols: &mut SymbolsTable<'ctx>,
    local: Local<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local_name);

    if let Instruction::LocalMut { value, .. } = local_value {
        let expression: BasicValueEnum =
            valuegen::generate_expression(module, builder, context, value, local_type, symbols);

        symbol.build_store(builder, expression);

        return;
    }

    if local_type.is_integer_type() {
        build_local_integer(module, builder, context, local, symbols);
        return;
    }

    if local_type.is_float_type() {
        build_local_float(module, builder, context, local, symbols);
        return;
    }

    if local_type.is_bool_type() {
        build_local_boolean(module, builder, context, local, symbols);
        return;
    }

    if local_type.is_ptr_type() {
        build_local_ptr(module, builder, context, local, symbols);
        return;
    }

    todo!()
}

fn build_local_ptr<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::generate_expression(
        module,
        builder,
        context,
        local_value,
        &Type::Ptr(None),
        symbols,
    );

    symbol.build_store(builder, expression);
}

fn build_local_structure<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local.0);

    if let Instruction::InitStruct { arguments, .. } = local_value {
        arguments.iter().for_each(|argument| {
            let argument_instruction: &Instruction = &argument.1;
            let argument_type: &Type = &argument.2;
            let argument_position: u32 = argument.3;

            let compiled_field: BasicValueEnum = valuegen::generate_expression(
                module,
                builder,
                context,
                argument_instruction,
                argument_type,
                symbols,
            );

            let field_in_struct: PointerValue = builder
                .build_struct_gep(
                    typegen::generate_type(context, local_type),
                    symbol.ptr,
                    argument_position,
                    "",
                )
                .unwrap();

            builder
                .build_store(field_in_struct, compiled_field)
                .unwrap();
        });

        return;
    }

    let expression: BasicValueEnum =
        valuegen::generate_expression(module, builder, context, local_value, local_type, symbols);

    symbol.build_store(builder, expression);
}

fn build_local_str<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local.0);

    let expression: BasicValueEnum =
        valuegen::generate_expression(module, builder, context, local_value, local.1, symbols);

    symbol.build_store(builder, expression);
}

fn build_local_integer<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(module, builder, context, local_value, local_type, symbols);

    symbol.build_store(builder, expression);
}

fn build_local_float<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(module, builder, context, local_value, local_type, symbols);

    symbol.build_store(builder, expression);
}

fn build_local_boolean<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    symbols: &mut SymbolsTable<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let symbol: AllocatedSymbol = symbols.get_allocated_symbol(local_name);

    let expression: BasicValueEnum =
        valuegen::generate_expression(module, builder, context, local_value, local_type, symbols);

    symbol.build_store(builder, expression);
}
