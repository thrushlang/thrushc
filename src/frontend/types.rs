use super::super::backend::compiler::instruction::Instruction;

pub type CodeLocation = (usize, (usize, usize));

pub type StructureFields<'instr> =
    Vec<(&'instr str, Instruction<'instr>, Instruction<'instr>, u32)>;

pub type TokenLexeme<'a> = &'a [u8];
