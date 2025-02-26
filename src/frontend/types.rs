use super::{super::backend::instruction::Instruction, lexer::DataTypes};

pub type StructFields<'instr> = Vec<(String, Instruction<'instr>, DataTypes, u32)>;
