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
    let local_type: &ThrushType = local.1;
    let local_value: &ThrushStatement = local.2;

    let attributes: &ThrushAttributes = local.3;

    context.alloc_local(local_name, local_type, attributes);

    if local_type.is_struct_type() {
        compile_local_structure(local, context);
        return;
    }

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::compile(context, local_value, Some(local_type));

    symbol.store(context, expression);
}

fn compile_local_structure<'ctx>(local: Local<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
    let local_value: &ThrushStatement = local.2;

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

            memory::store_anon(context, field_memory_address_position, expr_cast_type, expr);
        });
    }
}
