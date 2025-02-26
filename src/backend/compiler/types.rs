use super::{
    super::super::frontend::lexer::{DataTypes, TokenKind},
    Instruction,
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Instruction<'ctx>, &'ctx DataTypes);

pub type Variable<'ctx> = (&'ctx str, &'ctx DataTypes, &'ctx Instruction<'ctx>);

pub type Call<'ctx> = (&'ctx str, &'ctx DataTypes, &'ctx [Instruction<'ctx>]);

pub type Function<'ctx> = (
    &'ctx str,
    &'ctx [Instruction<'ctx>],
    Option<&'ctx Box<Instruction<'ctx>>>,
    &'ctx DataTypes,
    &'ctx bool,
);

pub type Struct<'ctx> = Vec<(String, DataTypes, u32)>;
