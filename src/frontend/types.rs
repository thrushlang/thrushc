use {super::super::backend::instruction::Instruction, super::lexer::DataTypes};

pub type StructFieldsParser<'instr> = Vec<(String, Instruction<'instr>, DataTypes, u32)>;
