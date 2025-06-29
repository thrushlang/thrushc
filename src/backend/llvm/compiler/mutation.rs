use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        ptrgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::ThrushType},
};

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, expr: &'ctx Ast) {
    if let Ast::Mut { source, value, .. } = expr {
        if let Some(any_reference) = &source.0 {
            let reference_name: &str = any_reference.0;

            let reference: &Ast = &any_reference.1;
            let cast_type: &ThrushType = reference.get_type_unwrapped();

            let symbol: SymbolAllocated = context.get_allocated_symbol(reference_name);

            let value: BasicValueEnum = valuegen::compile(context, value, Some(cast_type));

            symbol.store(context, value);

            return;
        }

        if let Some(expr) = &source.1 {
            let cast_type: &ThrushType = expr.get_type_unwrapped();

            let ptr: BasicValueEnum = ptrgen::compile(context, expr, None);
            let value: BasicValueEnum = valuegen::compile(context, value, Some(cast_type));

            memory::store_anon(context, ptr.into_pointer_value(), value);

            return;
        }

        logging::log(
            LoggingType::BackendBug,
            "The source of a mutation could not be obtained.",
        );
    }

    logging::log(LoggingType::BackendBug, "A mutation cannot be executed.");
}
