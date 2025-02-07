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

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Instruction<'ctx>, &'ctx DataTypes);

pub type Variable<'ctx> = (&'ctx str, &'ctx DataTypes, &'ctx Instruction<'ctx>);
