use super::super::super::frontend::types::StructFields;

use super::{
    Instruction,
    memory::MemoryFlag,
    objects::CompilerObjects,
    traits::{
        AttributesExtensions, CompilerStructureFieldsExtensions, MappedHeapedPointersExtension,
        MemoryFlagsBasics,
    },
    types::{
        CompilerAttributes, CompilerStructure, CompilerStructureFields, MappedHeapPointers,
        MemoryFlags,
    },
    utils,
};

use inkwell::{builder::Builder, context::Context, types::StructType, values::PointerValue};

impl AttributesExtensions for CompilerAttributes<'_> {
    fn contain_ffi_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ffi_attribute())
    }

    fn contain_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }
}

impl MemoryFlagsBasics for MemoryFlags {
    fn is_stack_allocated(&self) -> bool {
        self.iter().any(|flag| *flag == MemoryFlag::StackAllocated)
    }
}

impl CompilerStructureFieldsExtensions for StructFields<'_> {
    fn contain_recursive_structure_type(
        &self,
        compiler_objects: &CompilerObjects,
        structure_name: &str,
    ) -> bool {
        let structure: &CompilerStructure = compiler_objects.get_struct(structure_name);
        let structure_fields: &CompilerStructureFields = &structure.1;

        structure_fields
            .iter()
            .any(|field| field.1 == structure_name)
    }
}

impl CompilerStructureFieldsExtensions for CompilerStructureFields<'_> {
    fn contain_recursive_structure_type(
        &self,
        compiler_objects: &CompilerObjects,
        structure_name: &str,
    ) -> bool {
        let structure: &CompilerStructure = compiler_objects.get_struct(structure_name);
        let structure_fields: &CompilerStructureFields = &structure.1;

        structure_fields
            .iter()
            .any(|field| field.1 == structure_name)
    }
}

impl MappedHeapedPointersExtension<'_> for MappedHeapPointers<'_> {
    fn dealloc(
        &self,
        builder: &Builder,
        context: &Context,
        pointer: PointerValue,
        compiler_objects: &CompilerObjects,
    ) {
        self.iter()
            .filter(|mapped_pointer| !mapped_pointer.2)
            .for_each(|mapped_pointer| {
                let mapped_pointer_structure_name: &str = mapped_pointer.0;
                let mapped_pointer_index: u32 = mapped_pointer.1;

                let structure: &CompilerStructure =
                    compiler_objects.get_struct(mapped_pointer_structure_name);
                let structure_fields: &CompilerStructureFields = &structure.1;

                let pointer_type: StructType =
                    utils::build_struct_type_from_fields(context, structure_fields);

                let target_pointer: PointerValue = builder
                    .build_struct_gep(pointer_type, pointer, mapped_pointer_index, "")
                    .unwrap();

                let loaded_target_pointer: PointerValue = builder
                    .build_load(target_pointer.get_type(), target_pointer, "")
                    .unwrap()
                    .into_pointer_value();

                let _ = builder.build_free(loaded_target_pointer);
            });
    }
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Instruction::Integer(_, _, _), Instruction::Integer(_, _, _))
            | (Instruction::Float(_, _, _), Instruction::Float(_, _, _))
            | (Instruction::Str(_), Instruction::Str(_)) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}
