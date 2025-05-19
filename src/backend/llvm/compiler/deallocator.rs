use std::rc::Rc;

use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, PointerValue},
};

use crate::middle::types::{
    backend::llvm::types::SymbolsAllocated, frontend::parser::stmts::instruction::Instruction,
};

use super::{context::LLVMCodeGenContext, memory::SymbolAllocated};

pub fn dealloc_all(context: &LLVMCodeGenContext<'_, '_>, symbols_allocated: SymbolsAllocated) {
    symbols_allocated.iter().for_each(|any_symbol| {
        let symbol: &SymbolAllocated = any_symbol.1;
        symbol.dealloc(context);
    });

    self::destroy_calls(context);
}

pub fn dealloc(
    context: &LLVMCodeGenContext<'_, '_>,
    symbols_allocated: SymbolsAllocated,
    exclusion: Option<&Rc<Instruction>>,
) {
    if let Some(expression) = exclusion {
        if let Instruction::LocalRef { name, .. } = **expression {
            symbols_allocated
                .iter()
                .filter(|symbol| *symbol.0 != name)
                .for_each(|symbol| {
                    let symbol: &SymbolAllocated = symbol.1;
                    symbol.dealloc(context);
                });

            return;
        }

        self::dealloc_all(context, symbols_allocated);
        self::destroy_calls(context);
    }
}

fn destroy_calls(context: &LLVMCodeGenContext<'_, '_>) {
    for call in context.get_llvm_calls().iter() {
        let call_value: BasicValueEnum = call.1;

        let llvm_builder: &Builder = context.get_llvm_builder();

        if call_value.is_pointer_value() {
            let ptr: PointerValue = call_value.into_pointer_value();
            let _ = llvm_builder.build_free(ptr);
        }
    }
}
