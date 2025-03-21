use super::objects::CompilerObjects;

pub trait StructureBasics {
    fn contain_heaped_fields(&self, compile_objects: &CompilerObjects) -> bool;
}

pub trait AttributesExtensions {
    fn contain_ffi_attribute(&self) -> bool;
    fn contain_ignore_attribute(&self) -> bool;
}
