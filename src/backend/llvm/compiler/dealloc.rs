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
    }

    pub fn dealloc(&self, symbols_allocated: SymbolsAllocated, value: &Instruction) {
        if let Instruction::LocalRef { name, .. } = value {
            for symbol in symbols_allocated {
                let symbol_name = symbol.0;

                if symbol_name == name {
                    continue;
                }

                let symbol: &SymbolAllocated = symbol.1;
                symbol.dealloc(self.context);
            }
        }
    }
}
