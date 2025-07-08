use crate::frontend::{types::ast::metadata::staticvar::StaticMetadata, typesystem::types::Type};

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

pub trait StaticSymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_metadata(&self) -> StaticMetadata;
}
