use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::span::Span,
    types::frontend::{
        lexer::types::ThrushType,
        parser::stmts::types::{CompilerAttributes, CustomTypeFields, EnumFields},
    },
};

pub type Struct<'instr> = (
    &'instr str,
    Vec<(&'instr str, ThrushType, u32, Span)>,
    CompilerAttributes<'instr>,
    Methods<'instr>,
);

pub type Methods<'instr> = Vec<(&'instr str, ThrushType, Vec<ThrushType>)>;
pub type MethodDef<'instr> = &'instr (&'instr str, ThrushType, Vec<ThrushType>);

pub type Function<'instr> = (ThrushType, ParametersTypes, bool);

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
    Option<(&'instr str, usize)>,
    Option<(&'instr str, usize)>,
);

pub type CustomTypeSymbol<'ctx> = (CustomTypeFields<'ctx>, CompilerAttributes<'ctx>);
pub type EnumSymbol<'ctx> = (EnumFields<'ctx>, CompilerAttributes<'ctx>);
pub type ConstantSymbol<'instr> = (ThrushType, CompilerAttributes<'instr>);

pub type LLISymbol<'instr> = (ThrushType, Span);
pub type LocalSymbol<'instr> = (ThrushType, bool, bool, Span);
pub type ParameterSymbol<'instr> = (ThrushType, bool, bool, Span);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomTypeSymbol<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, ConstantSymbol<'instr>>;

pub type Parameters<'instr> = HashMap<&'instr str, ParameterSymbol<'instr>>;
pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, EnumSymbol<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type LLIs<'instr> = Vec<HashMap<&'instr str, LLISymbol<'instr>>>;
pub type Locals<'instr> = Vec<HashMap<&'instr str, LocalSymbol<'instr>>>;
