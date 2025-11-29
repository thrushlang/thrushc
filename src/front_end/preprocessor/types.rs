use crate::{
    core::diagnostic::span::Span,
    front_end::{preprocessor::signatures::ExternalSymbol, typesystem::types::Type},
};

use ahash::AHashMap as HashMap;

pub type FunctionParametersSignature = Vec<(Type, Span)>;
pub type EnumFieldsSignature = Vec<(Type, Span)>;

pub type FunctionParameterSignature = (Type, Span);
pub type EnumFieldSignature = (Type, Span);

pub type ExternalSymbols = Vec<ExternalSymbol>;

pub type FoundModuleSymbolId = (Option<String>, Option<String>);

pub type CustomTypeSymbol = Type;
pub type StructSymbol = Type;

pub type GlobalCustomTypes = HashMap<String, CustomTypeSymbol>;
pub type GlobalStructs = HashMap<String, StructSymbol>;
