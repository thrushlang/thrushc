use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    frontend::{lexer::span::Span, types::lexer::ThrushType},
};

use super::stmt::ThrushStatement;

pub type StructFields<'ctx> = (&'ctx str, Vec<(&'ctx str, ThrushType, u32, Span)>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, ThrushStatement<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, ThrushStatement<'ctx>);

pub type CustomTypeField<'ctx> = ThrushType;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'instr> = (
    &'instr str,
    Vec<(&'instr str, ThrushStatement<'instr>, ThrushType, u32)>,
);

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
