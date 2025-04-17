use super::{super::backend::compiler::instruction::Instruction, lexer::Type};

pub type CodeLocation = (usize, (usize, usize));
pub type StructFields<'instr> = Vec<(&'instr str, Instruction<'instr>, Type, u32)>;
pub type TokenLexeme<'a> = &'a [u8];
pub type ComplexType<'a> = (Type, &'a str);
