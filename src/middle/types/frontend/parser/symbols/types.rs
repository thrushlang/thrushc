use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::Span,
    middle::types::frontend::{
        lexer::types::ThrushType,
        parser::stmts::types::{CustomType, Enum, ThrushAttributes},
    },
};

pub type FoundSymbolId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
);

pub type Constant<'instr> = (ThrushType, ThrushAttributes<'instr>);

pub type Struct<'instr> = (
    &'instr str,
    Vec<(&'instr str, ThrushType, u32)>,
    ThrushAttributes<'instr>,
    Bindings<'instr>,
);

pub type Bindings<'instr> = Vec<(&'instr str, ThrushType, Vec<ThrushType>)>;
pub type Bind<'instr> = &'instr (&'instr str, ThrushType, Vec<ThrushType>);

pub type Function<'instr> = (ThrushType, Parameters, bool);

#[derive(Debug, Clone)]
pub struct Parameters(pub Vec<ThrushType>);

pub type Local<'instr> = (ThrushType, bool, bool, Span);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomType<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, Constant<'instr>>;

pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, Enum<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;
