#![allow(clippy::enum_variant_names)]

use super::super::super::frontend::lexer::Type;

use super::{
    instruction::Instruction,
    objects::CompilerObjects,
    types::{MappedHeapPointer, MappedHeapPointers, Structure, StructureFields},
};

use {
    ahash::{HashSet, HashSetExt},
    inkwell::{
        builder::Builder,
        types::BasicType,
        values::{BasicValue, BasicValueEnum, InstructionValue, PointerValue},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryFlag {
    StackAllocated,
    HeapAllocated,
    StaticAllocated,
}

#[derive(Debug, Clone, Copy)]
pub struct AllocatedObject<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub memory_flags: u8,
    pub kind: &'ctx Instruction<'ctx>,
}

impl<'ctx> AllocatedObject<'ctx> {
    pub fn alloc(
        ptr: PointerValue<'ctx>,
        flags: &[MemoryFlag],
        kind: &'ctx Instruction<'ctx>,
    ) -> Self {
        let mut memory_flags: u8 = 0;

        flags.iter().for_each(|flag| {
            memory_flags |= flag.to_bit();
        });

        Self {
            ptr,
            memory_flags,
            kind,
        }
    }

    pub fn load_from_memory<Type: BasicType<'ctx>>(
        &self,
        builder: &Builder<'ctx>,
        llvm_type: Type,
    ) -> BasicValueEnum<'ctx> {
        if self.has_flag(MemoryFlag::StackAllocated) {
            let load: BasicValueEnum = builder.build_load(llvm_type, self.ptr, "").unwrap();

            if let Some(load_instruction) = load.as_instruction_value() {
                let _ = load_instruction.set_alignment(8);
            }

            return load;
        }

        self.ptr.into()
    }

    pub fn dealloc(&self, builder: &Builder<'ctx>) {
        if self.has_flag(MemoryFlag::HeapAllocated) {
            let _ = builder.build_free(self.ptr);
        }
    }

    pub fn create_mapped_heaped_pointers(
        &self,
        compiler_objects: &'ctx CompilerObjects,
    ) -> MappedHeapPointers {
        if !self.kind.is_struct_type() {
            return HashSet::new();
        }

        let mut mapped_pointers: HashSet<MappedHeapPointer> = HashSet::with_capacity(10);

        if let Instruction::ComplexType(Type::Struct, structure_name) = self.kind {
            let structure: &Structure = compiler_objects.get_struct(structure_name);
            let structure_fields: &StructureFields = &structure.1;

            structure_fields
                .iter()
                .filter(|field| field.1.is_ptr_type())
                .for_each(|field| {
                    let field_position: u32 = field.2;

                    if let Instruction::ComplexType(Type::Struct, structure_name) = field.1 {
                        let structure: &Structure = compiler_objects.get_struct(structure_name);

                        let is_recursive: bool = structure
                            .1
                            .iter()
                            .filter(|field| field.1.is_struct_type())
                            .any(|field_recursive| field_recursive.1 == field.1);

                        mapped_pointers.insert((structure_name, field_position, is_recursive));
                    }
                });
        }

        mapped_pointers
    }

    pub fn build_store<Value: BasicValue<'ctx>>(&self, builder: &Builder<'ctx>, value: Value) {
        let store: InstructionValue = builder.build_store(self.ptr, value).unwrap();
        let _ = store.set_alignment(8);
    }

    pub fn has_flag(&self, flag: MemoryFlag) -> bool {
        (self.memory_flags & flag.to_bit()) == flag.to_bit()
    }
}

impl MemoryFlag {
    #[inline(always)]
    pub fn to_bit(&self) -> u8 {
        match self {
            MemoryFlag::StackAllocated => 1 << 0,
            MemoryFlag::HeapAllocated => 1 << 1,
            MemoryFlag::StaticAllocated => 1 << 2,
        }
    }
}
