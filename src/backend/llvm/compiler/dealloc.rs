use std::rc::Rc;

use crate::middle::instruction::Instruction;

use super::{context::CodeGenContext, memory::SymbolAllocated, types::SymbolsAllocated};

#[derive(Debug)]
pub struct Deallocator<'a, 'ctx> {
    context: &'a CodeGenContext<'a, 'ctx>,
}

impl<'a, 'ctx> Deallocator<'a, 'ctx> {
    pub fn new(context: &'a CodeGenContext<'a, 'ctx>) -> Self {
        Self { context }
    }

    pub fn dealloc_all(&self, symbols_allocated: SymbolsAllocated) {
        symbols_allocated.iter().for_each(|any_symbol| {
            let symbol: &SymbolAllocated = any_symbol.1;
            symbol.dealloc(self.context);
        });

        symbols_allocated.iter().for_each(|any_symbol| {
            let symbol: &SymbolAllocated = any_symbol.1;
            symbol.set_null(self.context);
        });
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

                symbols_allocated
                    .iter()
                    .filter(|symbol| *symbol.0 != name)
                    .for_each(|symbol| {
                        let symbol: &SymbolAllocated = symbol.1;
                        symbol.set_null(self.context);
                    });

                return;
            }

            self.dealloc_all(symbols_allocated);
        }
    }
}
