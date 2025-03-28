use super::{super::backend::compiler::instruction::Instruction, lexer::Type};

pub type StructFields<'instr> = Vec<(&'instr str, Instruction<'instr>, Type, u32)>;
pub type TokenLexeme<'a> = &'a [u8];
