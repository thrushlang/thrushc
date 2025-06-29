use ahash::AHashMap as HashMap;

use crate::{
    frontend::types::parser::stmts::types::{CustomTypeFields, EnumFields, ThrushAttributes},
    frontend::{lexer::span::Span, types::lexer::ThrushType},
};

pub type Struct<'parser> = (
    &'parser str,
    Vec<(&'parser str, ThrushType, u32, Span)>,
    ThrushAttributes<'parser>,
);

pub type Function<'parser> = (ThrushType, ParametersTypes, bool);
pub type AssemblerFunction<'parser> = (ThrushType, ParametersTypes, bool);

#[derive(Debug, Clone)]
pub struct ParametersTypes(pub Vec<ThrushType>);

/* ######################################################################


    PARSER - SYMBOLS


########################################################################*/

pub type FoundSymbolId<'parser> = (
    Option<&'parser str>,
    Option<&'parser str>,
    Option<&'parser str>,
    Option<(&'parser str, usize)>,
    Option<&'parser str>,
    Option<&'parser str>,
    Option<&'parser str>,
    Option<(&'parser str, usize)>,
    Option<(&'parser str, usize)>,
);

pub type CustomTypeSymbol<'ctx> = (CustomTypeFields<'ctx>, ThrushAttributes<'ctx>);
pub type EnumSymbol<'ctx> = (EnumFields<'ctx>, ThrushAttributes<'ctx>);
pub type ConstantSymbol<'parser> = (ThrushType, ThrushAttributes<'parser>);

pub type LLISymbol<'parser> = (ThrushType, Span);
pub type LocalSymbol<'parser> = (ThrushType, bool, bool, Span);
pub type ParameterSymbol<'parser> = (ThrushType, bool, bool, Span);

pub type CustomTypes<'parser> = HashMap<&'parser str, CustomTypeSymbol<'parser>>;
pub type LocalConstants<'parser> = Vec<HashMap<&'parser str, ConstantSymbol<'parser>>>;
pub type GlobalConstants<'parser> = HashMap<&'parser str, ConstantSymbol<'parser>>;

pub type Parameters<'parser> = HashMap<&'parser str, ParameterSymbol<'parser>>;
pub type Structs<'parser> = HashMap<&'parser str, Struct<'parser>>;
pub type Enums<'parser> = HashMap<&'parser str, EnumSymbol<'parser>>;
pub type Functions<'parser> = HashMap<&'parser str, Function<'parser>>;
pub type AssemblerFunctions<'parser> = HashMap<&'parser str, AssemblerFunction<'parser>>;

pub type LLIs<'parser> = Vec<HashMap<&'parser str, LLISymbol<'parser>>>;
pub type Locals<'parser> = Vec<HashMap<&'parser str, LocalSymbol<'parser>>>;
