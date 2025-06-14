use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        rawgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, expr: &'ctx ThrushStatement) {
    if let ThrushStatement::Mut { source, value, .. } = expr {
        let value_type: &ThrushType = value.get_type_unwrapped();

        if let Some(any_reference) = &source.0 {
            let reference_name: &str = any_reference.0;

            let symbol: SymbolAllocated = context.get_allocated_symbol(reference_name);

            let value: BasicValueEnum = valuegen::compile(context, value, None);

            symbol.store(context, value);

            return;
        }

        if let Some(expr) = &source.1 {
            let ptr: BasicValueEnum = rawgen::compile(context, expr, None);
            let value: BasicValueEnum = valuegen::compile(context, value, None);

            memory::store_anon(context, ptr.into_pointer_value(), value_type, value);

            return;
        }

        logging::log(LoggingType::Bug, "Could not get value of an mutation.");
    }

    logging::log(LoggingType::Bug, "Couldn't perform mutation.");
}
