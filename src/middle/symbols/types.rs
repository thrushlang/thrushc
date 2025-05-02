use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::Span,
    middle::{
        statement::{CustomType, Enum, ThrushAttributes},
        types::Type,
    },
};

pub type Constant<'instr> = (Type, ThrushAttributes<'instr>);

pub type Struct<'instr> = (
    &'instr str,
    Vec<(&'instr str, Type, u32)>,
    ThrushAttributes<'instr>,
    Bindings<'instr>,
);

pub type Bindings<'instr> = Vec<(&'instr str, Type, Vec<Type>)>;
pub type Bind<'instr> = &'instr (&'instr str, Type, Vec<Type>);

pub type Function<'instr> = (Type, Parameters, bool);

#[derive(Debug, Clone)]
pub struct Parameters(pub Vec<Type>);

pub type Local<'instr> = (Type, bool, bool, Span);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomType<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, Constant<'instr>>;

pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, Enum<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;
