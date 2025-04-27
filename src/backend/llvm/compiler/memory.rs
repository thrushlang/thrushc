#![allow(clippy::enum_variant_names)]

use inkwell::context::Context;
use inkwell::targets::TargetData;

use crate::middle::types::Type;

use super::{
    symbols::SymbolsTable,
    types::{MappedHeapPointer, MappedHeapPointers},
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
pub struct AllocatedSymbol<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub memory_flags: u8,
    pub kind: &'ctx Type,
}

impl<'ctx> AllocatedSymbol<'ctx> {
    pub fn alloc(ptr: PointerValue<'ctx>, flags: &[MemoryFlag], kind: &'ctx Type) -> Self {
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
        if self.has_flag(MemoryFlag::StackAllocated) || self.has_flag(MemoryFlag::StaticAllocated) {
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
        compiler_objects: &'ctx SymbolsTable,
    ) -> MappedHeapPointers {
        if !self.kind.is_struct_type() {
            return HashSet::new();
        }

        let mut mapped_pointers: HashSet<MappedHeapPointer> = HashSet::with_capacity(10);

        /*  if let Instruction::ComplexType(Type::Struct(_), structure_name, _) = self.kind {
            let fields: &StructureFields = compiler_objects.get_struct(structure_name).get_fields();

            fields
                .iter()
                .filter(|field| field.1.is_ptr_type())
                .for_each(|field| {
                    let field_position: u32 = field.2;

                    if let Instruction::ComplexType(Type::Struct(_), structure_name, _) = field.1 {
                        let structure: &Structure = compiler_objects.get_struct(structure_name);

                        let is_recursive: bool = structure
                            .1
                            .iter()
                            .filter(|field| field.1.is_struct_type())
                            .any(|field_recursive| field_recursive.1 == field.1);

                        mapped_pointers.insert((structure_name, field_position, is_recursive));
                    }
                });
        }*/

        mapped_pointers
    }

    pub fn is_stack_allocated(&self) -> bool {
        self.has_flag(MemoryFlag::StackAllocated)
    }

    pub fn is_heap_allocated(&self) -> bool {
        self.has_flag(MemoryFlag::HeapAllocated)
    }

    pub fn is_static_allocated(&self) -> bool {
        self.has_flag(MemoryFlag::StaticAllocated)
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

pub fn generate_site_allocation_flag(
    context: &Context,
    target_data: &TargetData,
    kind: &Type,
) -> MemoryFlag {
    let mut alloc_site_memory_flag: MemoryFlag = MemoryFlag::StackAllocated;

    if kind.is_struct_type() && kind.is_recursive_type()
        || kind.llvm_exceeds_stack(context, target_data) >= 120
    {
        alloc_site_memory_flag = MemoryFlag::HeapAllocated;
    }

    alloc_site_memory_flag
}
