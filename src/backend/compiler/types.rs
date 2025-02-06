use super::{
    super::super::frontend::lexer::{DataTypes, TokenKind},
    Instruction,
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
    &'ctx DataTypes,
);
