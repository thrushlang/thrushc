use super::super::super::frontend::lexer::Type;

pub trait StructFieldsExtensions {
    fn get_type(&self) -> Type;
}

pub trait ConstructorExtensions {
    fn get_type(&self) -> Type;
}

pub trait AttributesExtensions {
    fn contain_ffi_attribute(&self) -> bool;
    fn contain_ignore_attribute(&self) -> bool;
    fn contain_public_attribute(&self) -> bool;
}

pub trait MemoryFlagsBasics {
    fn is_stack_allocated(&self) -> bool;
}
