use crate::{
    backend::llvm::compiler::valuegen::CompileChanges,
    frontend::types::{
        lexer::ThrushType, parser::stmts::types::ThrushAttributes, representations::Local,
    },
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
    let local_name: &'ctx str = local.0;
    let local_type: &ThrushType = local.1;
    let local_value: &'ctx ThrushStatement<'ctx> = local.2;
    let attributes: &ThrushAttributes = local.3;

    context.alloc_local(local_name, local_type, attributes);

    if local_type.is_struct_type() {
        compile_local_structure(local, context);
        return;
    }

    let symbol: SymbolAllocated = context.get_allocated_symbol(local.0);

    let expression: BasicValueEnum = valuegen::compile(
        context,
        local_value,
        local_type,
        CompileChanges::new(false, true),
    );

    symbol.store(context, expression);
}

fn compile_local_structure<'ctx>(local: Local<'ctx>, context: &mut LLVMCodeGenContext<'_, 'ctx>) {
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

            let expr: BasicValueEnum =
                valuegen::compile(context, expr, expr_type, CompileChanges::new(false, true));

            let field_memory_address_position: PointerValue =
                symbol.gep_struct(llvm_context, llvm_builder, expr_index);

            memory::store_anon(context, field_memory_address_position, expr_type, expr);
        });
    } else {
        let expression: BasicValueEnum = valuegen::compile(
            context,
            local_value,
            local_type,
            CompileChanges::new(false, true),
        );

        symbol.store(context, expression);
    }
}
