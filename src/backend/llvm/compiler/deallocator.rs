use std::rc::Rc;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, PointerValue},
};

use crate::middle::types::{
    backend::llvm::types::SymbolsAllocated,
    frontend::{lexer::types::ThrushType, parser::stmts::instruction::Instruction},
};

use super::{context::LLVMCodeGenContext, memory::SymbolAllocated};

#[derive(Debug)]
pub struct Deallocator<'a, 'ctx> {
    context: &'a LLVMCodeGenContext<'a, 'ctx>,
}

impl<'a, 'ctx> Deallocator<'a, 'ctx> {
    pub fn new(context: &'a LLVMCodeGenContext<'a, 'ctx>) -> Self {
        Self { context }
    }

    pub fn dealloc_all(&self, symbols_allocated: SymbolsAllocated) {
        symbols_allocated.iter().for_each(|any_symbol| {
            let symbol: &SymbolAllocated = any_symbol.1;
            symbol.dealloc(self.context);
        });

        self.destroy_calls();
    }

    pub fn dealloc(
        &self,
        symbols_allocated: SymbolsAllocated,
        expression: Option<&Rc<Instruction>>,
    ) {
        if let Some(expression) = expression {
            if let Instruction::LocalRef { name, .. } = **expression {
                symbols_allocated
                    .iter()
                    .filter(|symbol| *symbol.0 != name)
                    .for_each(|symbol| {
                        let symbol: &SymbolAllocated = symbol.1;
                        symbol.dealloc(self.context);
                    });

                return;
            }

            self.dealloc_all(symbols_allocated);
            self.destroy_calls();
        }
    }

    fn destroy_calls(&self) {
        self.context.get_llvm_calls().iter().for_each(|call| {
            let call_type: &ThrushType = call.0;
            let call_value: BasicValueEnum = call.1;

            let llvm_context: &Context = self.context.get_llvm_context();
            let llvm_builder: &Builder = self.context.get_llvm_builder();

            let is_heap_allocated: bool =
                call_type.is_heap_allocated(llvm_context, &self.context.target_data);

            if !is_heap_allocated {
                return;
            }

            if call_value.is_pointer_value() {
                let ptr: PointerValue = call_value.into_pointer_value();
                let _ = llvm_builder.build_free(ptr);
            }
        });
    }
}
