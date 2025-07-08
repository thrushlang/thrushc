use crate::frontend::typesystem::types::Type;

pub trait LocalSymbolExtensions {
    fn is_mutable(&self) -> bool;
    fn get_type(&self) -> Type;
}

pub trait ConstantSymbolExtensions {
    fn get_type(&self) -> Type;
}

pub trait LLISymbolExtensions {
    fn get_type(&self) -> Type;
}

pub trait FunctionExtensions {
    fn get_type(&self) -> Type;
}
