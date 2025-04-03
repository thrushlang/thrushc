use inkwell::builder::Builder;

use super::{
    instruction::Instruction,
    memory::{AllocatedObject, MemoryFlag},
    types::AllocatedObjects,
};

#[derive(Debug)]
pub struct Deallocator<'ctx> {
    builder: &'ctx Builder<'ctx>,
    objects: &'ctx AllocatedObjects<'ctx>,
}

impl<'ctx> Deallocator<'ctx> {
    pub fn new(builder: &'ctx Builder<'ctx>, objects: &'ctx AllocatedObjects<'ctx>) -> Self {
        Self { builder, objects }
    }

    pub fn dealloc_all(&self) {
        let heaped_objects: Vec<(&&str, &AllocatedObject)> = self.obtain_heap_objects("");

        heaped_objects.iter().for_each(|heap_object| {
            let allocated_object: &AllocatedObject = heap_object.1;
            let _ = self.builder.build_free(allocated_object.ptr);
        });
    }

    pub fn dealloc(&self, return_instruction: &Instruction) {
        if let Instruction::LocalRef { name, .. } = return_instruction {
            let heaped_objects: Vec<(&&str, &AllocatedObject)> = self.obtain_heap_objects(name);
        }

        todo!()
    }

    fn obtain_heap_objects(&self, except: &str) -> Vec<(&&str, &AllocatedObject)> {
        self.objects
            .iter()
            .filter(|object| object.1.has_flag(MemoryFlag::HeapAllocated) && *object.0 != except)
            .collect::<Vec<_>>()
    }
}
