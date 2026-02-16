use thrustc_ast::{
    data::EnumData,
    metadata::{FunctionParameterMetadata, LocalMetadata, StaticMetadata},
};
use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_typesystem::{Type, modificators::StructureTypeModificator};

use ahash::AHashMap as HashMap;

pub type Struct<'parser> = (
    &'parser str,
    Vec<(&'parser str, Type, u32, Span)>,
    ThrustAttributes,
    StructureTypeModificator,
    Span,
);

pub type Function<'parser> = (Type, FunctionParametersTypes, bool);
pub type AssemblerFunction<'parser> = (Type, AssemblerFunctionParametersTypes, bool);
pub type Intrinsic<'parser> = (Type, IntrinsicParametersTypes, bool);

#[derive(Debug, Clone)]
pub struct FunctionParametersTypes(pub Vec<Type>);
#[derive(Debug, Clone)]
pub struct AssemblerFunctionParametersTypes(pub Vec<Type>);
#[derive(Debug, Clone)]
pub struct IntrinsicParametersTypes(pub Vec<Type>);

pub type FoundSymbolId<'parser> = (
    Option<(&'parser str, usize)>,
    Option<&'parser str>,
    Option<(&'parser str, usize)>,
    Option<(&'parser str, usize)>,
    Option<(&'parser str, usize)>,
    Option<(&'parser str, usize)>,
    Option<&'parser str>,
    Option<&'parser str>,
    Option<(&'parser str, usize)>,
    Option<(&'parser str, usize)>,
    Option<&'parser str>,
);

pub type CustomTypeSymbol<'ctx> = (Type, ThrustAttributes);
pub type EnumSymbol<'ctx> = (EnumData<'ctx>, ThrustAttributes);
pub type StaticSymbol<'parser> = (Type, StaticMetadata, ThrustAttributes);
pub type ConstantSymbol<'parser> = (Type, ThrustAttributes);

pub type LLISymbol<'parser> = (Type, Span);
pub type LocalSymbol<'parser> = (Type, LocalMetadata, Span);
pub type ParameterSymbol<'parser> = (Type, FunctionParameterMetadata, Span);

pub type GlobalCustomTypes<'parser> = HashMap<&'parser str, CustomTypeSymbol<'parser>>;
pub type LocalCustomTypes<'parser> = Vec<HashMap<&'parser str, CustomTypeSymbol<'parser>>>;

pub type GlobalStructs<'parser> = HashMap<&'parser str, Struct<'parser>>;
pub type LocalStructs<'parser> = Vec<HashMap<&'parser str, Struct<'parser>>>;

pub type LocalStatics<'parser> = Vec<HashMap<&'parser str, StaticSymbol<'parser>>>;
pub type GlobalStatics<'parser> = HashMap<&'parser str, StaticSymbol<'parser>>;

pub type LocalConstants<'parser> = Vec<HashMap<&'parser str, ConstantSymbol<'parser>>>;
pub type GlobalConstants<'parser> = HashMap<&'parser str, ConstantSymbol<'parser>>;

pub type GlobalEnums<'parser> = HashMap<&'parser str, EnumSymbol<'parser>>;
pub type LocalEnums<'parser> = Vec<HashMap<&'parser str, EnumSymbol<'parser>>>;

pub type Parameters<'parser> = HashMap<&'parser str, ParameterSymbol<'parser>>;

pub type Intrinsics<'parser> = HashMap<&'parser str, Intrinsic<'parser>>;

pub type Functions<'parser> = HashMap<&'parser str, Function<'parser>>;
pub type AssemblerFunctions<'parser> = HashMap<&'parser str, AssemblerFunction<'parser>>;

pub type LLIs<'parser> = Vec<HashMap<&'parser str, LLISymbol<'parser>>>;
pub type Locals<'parser> = Vec<HashMap<&'parser str, LocalSymbol<'parser>>>;
