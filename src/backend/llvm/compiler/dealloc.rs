use crate::middle::instruction::Instruction;

use super::{
    memory::{AllocatedSymbol, MemoryFlag},
    symbols::SymbolsTable,
    types::{AllocatedSymbols, MappedHeapPointers},
};

use inkwell::{builder::Builder, context::Context};

#[derive(Debug)]
pub struct Deallocator<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    objects: AllocatedSymbols<'ctx>,
}

impl<'ctx> Deallocator<'ctx> {
    pub fn new(
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        objects: AllocatedSymbols<'ctx>,
    ) -> Self {
        Self {
            builder,
            context,
            objects,
        }
    }

    pub fn dealloc_all(&self, symbols: &SymbolsTable) {
        let heaped_objects: Vec<(&&str, &AllocatedSymbol)> = self.obtain_heap_objects("");

        heaped_objects.iter().for_each(|heap_object| {
            let allocated_object: &AllocatedSymbol = heap_object.1;

            let mapped_heaped_pointers: MappedHeapPointers =
                allocated_object.create_mapped_heaped_pointers(symbols);

            /*mapped_heaped_pointers.dealloc(
                self.builder,
                self.context,
                allocated_object.ptr,
                symbols,
            );*/
            allocated_object.dealloc(self.builder);
        });
    }

    pub fn dealloc(&self, value: &Instruction, symbols: &SymbolsTable) {
        if let Instruction::LocalRef { name, .. } = value {
            let heaped_objects: Vec<(&&str, &AllocatedSymbol)> = self.obtain_heap_objects(name);

            heaped_objects.iter().for_each(|heap_object| {
                let allocated_object: &AllocatedSymbol = heap_object.1;

                let mapped_heaped_pointers: MappedHeapPointers =
                    allocated_object.create_mapped_heaped_pointers(symbols);

                /*mapped_heaped_pointers.dealloc(
                    self.builder,
                    self.context,
                    allocated_object.ptr,
                    symbols,
                );*/

                allocated_object.dealloc(self.builder);
            });
        }
    }

    fn obtain_heap_objects(&self, except: &str) -> Vec<(&&str, &AllocatedSymbol)> {
        self.objects
            .iter()
            .filter(|object| object.1.has_flag(MemoryFlag::HeapAllocated) && *object.0 != except)
            .collect::<Vec<_>>()
    }
}
