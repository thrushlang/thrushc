use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    middle::types::frontend::lexer::types::ThrushType,
};

use super::instruction::Instruction;

pub type StructFields<'ctx> = (&'ctx str, Vec<(&'ctx str, ThrushType, u32)>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, Instruction<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Instruction<'ctx>);

pub type CustomTypeField<'ctx> = ThrushType;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'instr> = (
    &'instr str,
    Vec<(&'instr str, Instruction<'instr>, ThrushType, u32)>,
);

pub type CompilerAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
