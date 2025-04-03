use super::{
    super::super::frontend::types::StructFields,
    memory::MemoryFlag,
    objects::CompilerObjects,
    traits::{AttributesExtensions, CompilerStructureFieldsExtensions, MemoryFlagsBasics},
    types::{CompilerAttributes, CompilerStructure, CompilerStructureFields, MemoryFlags},
};

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
