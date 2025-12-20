use crate::core::diagnostic::span::Span;

use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::ast::metadata::staticvar::StaticMetadata;
use crate::front_end::types::parser::stmts::types::EnumFields;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

use ahash::AHashMap as HashMap;

pub type Struct<'parser> = (
    &'parser str,
    Vec<(&'parser str, Type, u32, Span)>,
    ThrushAttributes,
    StructureTypeModificator,
    Span,
);

pub type Function<'parser> = (Type, ParametersTypes, bool);
pub type AssemblerFunction<'parser> = (Type, ParametersTypes, bool);
pub type Intrinsic<'parser> = (Type, ParametersTypes, bool);

#[derive(Debug, Clone)]
pub struct ParametersTypes(pub Vec<Type>);

/* ######################################################################


    PARSER - SYMBOLS


########################################################################*/

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

pub type CustomTypeSymbol<'ctx> = (Type, ThrushAttributes);
pub type EnumSymbol<'ctx> = (EnumFields<'ctx>, ThrushAttributes);
pub type StaticSymbol<'parser> = (Type, StaticMetadata, ThrushAttributes);
pub type ConstantSymbol<'parser> = (Type, ThrushAttributes);

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
