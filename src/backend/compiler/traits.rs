use super::objects::CompilerObjects;

pub trait StructureBasics {
    fn contain_heaped_fields(&self, compile_objects: &CompilerObjects) -> bool;
}
