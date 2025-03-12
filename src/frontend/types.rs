use super::{super::backend::instruction::Instruction, lexer::DataTypes};

pub type StructFields<'instr> = Vec<(&'instr str, Instruction<'instr>, DataTypes, u32)>;
