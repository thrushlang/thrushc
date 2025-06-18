use crate::frontend::types::{
    lexer::ThrushType, parser::stmts::types::ThrushAttributes, representations::Local,
};

use super::{
    ThrushStatement,
    context::LLVMCodeGenContext,
    memory::{self, SymbolAllocated},
    valuegen,
};

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};

pub fn compile<'ctx>(local: Local<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_name: &str = local.0;
    let ascii_name: &str = local.1;
    let local_type: &ThrushType = local.2;
    let local_value: &ThrushStatement = local.3;

    let attributes: &ThrushAttributes = local.4;

    context.alloc_local(local_name, ascii_name, local_type, attributes);

    context.set_site_allocation(memory::get_memory_site_allocation_from_attributes(
        attributes,
    ));

    if local_type.is_struct_type() {
        self::compile_local_structure(local, context);
        context.reset_site_allocation();

        return;
    }

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::compile(context, local_value, Some(local_type));

    symbol.store(context, expression);

    context.reset_site_allocation();
}

fn compile_local_structure<'ctx>(local: Local<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &ThrushStatement = local.3;

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    if let ThrushStatement::Constructor { arguments, .. } = local_value {
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_context: &Context = context.get_llvm_context();

        let expressions: &[(&str, ThrushStatement, ThrushType, u32)] = &arguments.1;

        expressions.iter().for_each(|argument| {
            let expr: &ThrushStatement = &argument.1;
            let expr_index: u32 = argument.3;

            let expr_cast_type: &ThrushType = &argument.2;

            let expr: BasicValueEnum = valuegen::compile(context, expr, Some(expr_cast_type));

            let field_memory_address_position: PointerValue =
                symbol.gep_struct(llvm_context, llvm_builder, expr_index);

            memory::store_anon(context, field_memory_address_position, expr);
        });
    }
}
