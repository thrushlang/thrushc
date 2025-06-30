use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    frontend::{
        lexer::span::Span,
        types::{ast::Ast, lexer::ThrushType},
    },
};

pub type StructFields<'ctx> = (&'ctx str, Vec<(&'ctx str, ThrushType, u32, Span)>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, Ast<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Ast<'ctx>);

pub type CustomTypeField<'ctx> = ThrushType;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'ctx> = Vec<(&'ctx str, Ast<'ctx>, ThrushType, u32)>;

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
