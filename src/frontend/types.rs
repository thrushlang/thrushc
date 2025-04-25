use super::{super::backend::compiler::instruction::Instruction, lexer::Type};

pub type CodeLocation = (usize, (usize, usize));

pub type Constructor<'instr> = Vec<(&'instr str, Instruction<'instr>, Type, u32)>;

pub type TokenLexeme<'a> = &'a [u8];
