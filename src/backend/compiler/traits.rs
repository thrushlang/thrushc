use super::objects::CompilerObjects;

pub trait StructureBasics {
    fn contain_heaped_fields(&self, compile_objects: &CompilerObjects) -> bool;
}

pub trait CompilerStructureFieldsExtensions {
    fn contain_recursive_structure_type(
        &self,
        compiler_objects: &CompilerObjects,
        structure: &str,
    ) -> bool;
}

pub trait AttributesExtensions {
    fn contain_ffi_attribute(&self) -> bool;
    fn contain_ignore_attribute(&self) -> bool;
}

pub trait MemoryFlagsBasics {
    fn is_stack_allocated(&self) -> bool;
}
