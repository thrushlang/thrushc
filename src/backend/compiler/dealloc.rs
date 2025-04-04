use {
    super::{
        instruction::Instruction,
        memory::{AllocatedObject, MemoryFlag},
        objects::CompilerObjects,
        traits::MappedHeapedPointersExtension,
        types::{AllocatedObjects, MappedHeapedPointers},
    },
    inkwell::{builder::Builder, context::Context},
};

#[derive(Debug)]
pub struct Deallocator<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    objects: AllocatedObjects<'ctx>,
}

impl<'ctx> Deallocator<'ctx> {
    pub fn new(
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        objects: AllocatedObjects<'ctx>,
    ) -> Self {
        Self {
            builder,
            context,
            objects,
        }
    }

    pub fn dealloc_all(&self, compiler_objects: &CompilerObjects) {
        let heaped_objects: Vec<(&&str, &AllocatedObject)> = self.obtain_heap_objects("");

        heaped_objects.iter().for_each(|heap_object| {
            let allocated_object: &AllocatedObject = heap_object.1;

            let mapped_heaped_pointers: MappedHeapedPointers =
                allocated_object.generate_mapped_heaped_pointers(compiler_objects);

            mapped_heaped_pointers.dealloc(
                self.builder,
                self.context,
                allocated_object.ptr,
                compiler_objects,
            );
            allocated_object.dealloc(self.builder);
        });
    }

    pub fn dealloc(&self, return_instruction: &Instruction, compiler_objects: &CompilerObjects) {
        if let Instruction::LocalRef { name, .. } = return_instruction {
            let heaped_objects: Vec<(&&str, &AllocatedObject)> = self.obtain_heap_objects(name);

            heaped_objects.iter().for_each(|heap_object| {
                let allocated_object: &AllocatedObject = heap_object.1;

                let mapped_heaped_pointers: MappedHeapedPointers =
                    allocated_object.generate_mapped_heaped_pointers(compiler_objects);

                mapped_heaped_pointers.dealloc(
                    self.builder,
                    self.context,
                    allocated_object.ptr,
                    compiler_objects,
                );
                allocated_object.dealloc(self.builder);
            });
        }
    }

    fn obtain_heap_objects(&self, except: &str) -> Vec<(&&str, &AllocatedObject)> {
        self.objects
            .iter()
            .filter(|object| object.1.has_flag(MemoryFlag::HeapAllocated) && *object.0 != except)
            .collect::<Vec<_>>()
    }
}
