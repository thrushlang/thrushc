use crate::middle::instruction::Instruction;

use super::{memory::SymbolAllocated, symbols::SymbolsTable, types::SymbolsAllocated};

#[derive(Debug)]
pub struct Deallocator<'a, 'ctx> {
    symbols: &'a SymbolsTable<'a, 'ctx>,
}

impl<'a, 'ctx> Deallocator<'a, 'ctx> {
    pub fn new(symbols: &'a SymbolsTable<'a, 'ctx>) -> Self {
        Self { symbols }
    }

    pub fn dealloc_all(&self, symbols_allocated: SymbolsAllocated) {
        symbols_allocated.iter().for_each(|any_symbol| {
            let symbol: &SymbolAllocated = any_symbol.1;
            symbol.dealloc(self.symbols);
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
                symbol.dealloc(self.symbols);
            }
        }
    }
}
