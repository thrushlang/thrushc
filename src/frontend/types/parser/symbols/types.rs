use ahash::AHashMap as HashMap;

use crate::{
    frontend::types::parser::stmts::types::{CustomTypeFields, EnumFields, ThrushAttributes},
    frontend::{lexer::span::Span, types::lexer::ThrushType},
};

pub type Struct<'instr> = (
    &'instr str,
    Vec<(&'instr str, ThrushType, u32, Span)>,
    ThrushAttributes<'instr>,
);

pub type Function<'instr> = (ThrushType, ParametersTypes, bool);
pub type AssemblerFunction<'instr> = (ThrushType, ParametersTypes, bool);

#[derive(Debug, Clone)]
pub struct ParametersTypes(pub Vec<ThrushType>);

/* ######################################################################


    PARSER - SYMBOLS


########################################################################*/

pub type FoundSymbolId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
    Option<(&'instr str, usize)>,
);

pub type CustomTypeSymbol<'ctx> = (CustomTypeFields<'ctx>, ThrushAttributes<'ctx>);
pub type EnumSymbol<'ctx> = (EnumFields<'ctx>, ThrushAttributes<'ctx>);
pub type ConstantSymbol<'instr> = (ThrushType, ThrushAttributes<'instr>);

pub type LLISymbol<'instr> = (ThrushType, Span);
pub type LocalSymbol<'instr> = (ThrushType, bool, bool, Span);
pub type ParameterSymbol<'instr> = (ThrushType, bool, bool, Span);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomTypeSymbol<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, ConstantSymbol<'instr>>;

pub type Parameters<'instr> = HashMap<&'instr str, ParameterSymbol<'instr>>;
pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, EnumSymbol<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;
pub type AssemblerFunctions<'instr> = HashMap<&'instr str, AssemblerFunction<'instr>>;

pub type LLIs<'instr> = Vec<HashMap<&'instr str, LLISymbol<'instr>>>;
pub type Locals<'instr> = Vec<HashMap<&'instr str, LocalSymbol<'instr>>>;
