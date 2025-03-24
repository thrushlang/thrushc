use super::{
    super::super::backend::instruction::CompilerAttribute,
    super::super::frontend::lexer::{TokenKind, Type},
    Instruction,
};

pub type BinaryOp<'ctx> = (
    &'ctx Instruction<'ctx>,
    &'ctx TokenKind,
    &'ctx Instruction<'ctx>,
);

pub type UnaryOp<'ctx> = (&'ctx TokenKind, &'ctx Instruction<'ctx>, &'ctx Type);

pub type Local<'ctx> = (&'ctx str, &'ctx Type, &'ctx Instruction<'ctx>);

pub type Call<'ctx> = (&'ctx str, &'ctx Type, &'ctx [Instruction<'ctx>]);

pub type Function<'ctx> = (
    &'ctx str,
    &'ctx Type,
    &'ctx [Instruction<'ctx>],
    Option<&'ctx Box<Instruction<'ctx>>>,
    &'ctx [CompilerAttribute<'ctx>],
);

pub type Struct<'ctx> = Vec<(&'ctx str, Type, u32)>;
pub type StructField<'ctx> = (&'ctx str, Type, u32);

pub type CompilerAttributes<'ctx> = Vec<CompilerAttribute<'ctx>>;
