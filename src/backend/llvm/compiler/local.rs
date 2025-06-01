use crate::types::{backend::llvm::types::LLVMLocal, frontend::lexer::types::ThrushType};

use super::{
    ThrushStatement,
    context::{LLVMCodeGenContext, LLVMCodeGenContextPosition},
    memory::{self, SymbolAllocated},
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
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::build(context, local_value, local.1);

    symbol.store(context, expression);
}

fn build_local_ptr<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::build(context, local_value, &ThrushType::Ptr(None));

    symbol.store(context, expression);
}

fn build_local_str<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::build(context, local_value, local.1);

    symbol.store(context, expression);
}

fn build_local_integer<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(context, local_value, local_type);

    symbol.store(context, expression);
}

fn build_local_float<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(context, local_value, local_type);

    symbol.store(context, expression);
}

fn build_local_boolean<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(context, local_value, local_type);

    symbol.store(context, expression);
}

fn build_local_structure<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_type: &ThrushType = local.1;
    let local_value: &ThrushStatement = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    if let ThrushStatement::Constructor { arguments, .. } = local_value {
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_context: &Context = context.get_llvm_context();

        let exprs: &[(&str, ThrushStatement<'_>, ThrushType, u32)] = &arguments.1;

        exprs.iter().for_each(|argument| {
            let expr: &ThrushStatement = &argument.1;
            let expr_type: &ThrushType = &argument.2;
            let expr_index: u32 = argument.3;

            let expr: BasicValueEnum = valuegen::build(context, expr, expr_type);

            let field_memory_address_position: PointerValue =
                symbol.gep_struct(llvm_context, llvm_builder, expr_index);

            memory::store_anon(context, field_memory_address_position, expr);
        });
    } else {
        let expression: BasicValueEnum = valuegen::build(context, local_value, local_type);

        symbol.store(context, expression);
    }
}
