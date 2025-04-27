use ahash::AHashMap as HashMap;

use crate::middle::{
    statement::{CustomType, Enum, ThrushAttributes},
    types::Type,
};

pub type Constant<'instr> = (Type, ThrushAttributes<'instr>);

pub type Struct<'instr> = (Vec<(&'instr str, Type, u32)>, ThrushAttributes<'instr>);

pub type Function<'instr> = (Type, Vec<Type>, bool);
pub type Local<'instr> = (Type, bool);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomType<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, Constant<'instr>>;

pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, Enum<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;
