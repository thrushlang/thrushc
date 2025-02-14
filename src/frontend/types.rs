use {super::super::backend::instruction::Instruction, super::lexer::DataTypes};

pub type StructFields<'instr> = Vec<(String, Instruction<'instr>, DataTypes, u32)>;
