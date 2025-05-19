use crate::middle::types::{backend::llvm::types::LLVMLocal, frontend::lexer::types::ThrushType};

use super::{
    Instruction,
    context::{LLVMCodeGenContext, LLVMCodeGenContextPosition},
    memory::{self, AllocSite, SymbolAllocated},
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

    let expression: BasicValueEnum = valuegen::build(local_value, local.1, context);

    symbol.store(context, expression);
}

fn build_local_ptr<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::build(local_value, &ThrushType::Ptr(None), context);

    symbol.store(context, expression);
}

fn build_local_str<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::build(local_value, local.1, context);

    symbol.store(context, expression);
}

fn build_local_integer<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_float<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_boolean<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local_name);

    let expression: BasicValueEnum = valuegen::build(local_value, local_type, context);

    symbol.store(context, expression);
}

fn build_local_structure<'ctx>(local: LLVMLocal<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_type: &ThrushType = local.1;
    let local_value: &Instruction = local.2;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    if let Instruction::Constructor { arguments, .. } = local_value {
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_context: &Context = context.get_llvm_context();

        let exprs: &[(&str, Instruction<'_>, ThrushType, u32)] = &arguments.1;

        exprs.iter().for_each(|argument| {
            let expr: &Instruction = &argument.1;
            let expr_type: &ThrushType = &argument.2;
            let expr_index: u32 = argument.3;

            let mut expr: BasicValueEnum = valuegen::build(expr, expr_type, context);

            if expr_type.is_heap_allocated(llvm_context, &context.target_data)
                && expr_type.is_me_type()
            {
                let src_ptr: PointerValue = expr.into_pointer_value();

                if !src_ptr.is_null() {
                    let dest_ptr: PointerValue =
                        memory::alloc(AllocSite::Heap, context, local_type);

                    memory::memcpy(context, dest_ptr, src_ptr, local_type);

                    expr = dest_ptr.into();
                }
            }

            let field_memory_address_position: PointerValue =
                symbol.gep_struct(llvm_context, llvm_builder, expr_index);

            memory::store_anon(context, field_memory_address_position, expr);
        });
    } else {
        let expression: BasicValueEnum = valuegen::build(local_value, local_type, context);

        symbol.store(context, expression);
    }
}
