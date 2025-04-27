use crate::middle::instruction::Instruction;

use super::{
    memory::SymbolAllocated,
    symbols::SymbolsTable,
    types::{MappedHeapPointers, SymbolsAllocated},
};

use inkwell::{builder::Builder, context::Context};

#[derive(Debug)]
pub struct Deallocator<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    objects: SymbolsAllocated<'ctx>,
}

impl<'ctx> Deallocator<'ctx> {
    pub fn new(
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        objects: SymbolsAllocated<'ctx>,
    ) -> Self {
        Self {
            builder,
            context,
            objects,
        }
    }

    pub fn dealloc_all(&self, symbols: &SymbolsTable) {
        let heaped_objects: Vec<(&&str, &SymbolAllocated)> = self.obtain_heap_objects("");

        heaped_objects.iter().for_each(|heap_object| {
            let allocated_object: &SymbolAllocated = heap_object.1;

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
            let heaped_objects: Vec<(&&str, &SymbolAllocated)> = self.obtain_heap_objects(name);

            heaped_objects.iter().for_each(|heap_object| {
                let allocated_object: &SymbolAllocated = heap_object.1;

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

    fn obtain_heap_objects(&self, except: &str) -> Vec<(&&str, &SymbolAllocated)> {
        self.objects
            .iter()
            .filter(|object| object.1.get_type().is_recursive_type() && *object.0 != except)
            .collect::<Vec<_>>()
    }
}
