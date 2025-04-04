use {
    super::objects::CompilerObjects,
    inkwell::{builder::Builder, context::Context, values::PointerValue},
};

pub trait MappedHeapedPointersExtension<'ctx> {
    fn dealloc(
        &self,
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        pointer: PointerValue<'ctx>,
        compiler_objects: &CompilerObjects,
    );
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
