use crate::frontend::{
    types::ast::metadata::{
        fnparam::FunctionParameterMetadata, local::LocalMetadata, staticvar::StaticMetadata,
    },
    typesystem::types::Type,
};

pub trait LocalSymbolExtensions {
    fn get_metadata(&self) -> LocalMetadata;
    fn get_type(&self) -> Type;
}

pub trait StaticSymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_metadata(&self) -> StaticMetadata;
}

pub trait FunctionParameterSymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_metadata(&self) -> FunctionParameterMetadata;
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
