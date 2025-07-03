use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    frontend::{
        lexer::span::Span,
        types::{ast::Ast, lexer::Type},
    },
};

pub type StructFields<'ctx> = (&'ctx str, Vec<(&'ctx str, Type, u32, Span)>);

pub type EnumFields<'ctx> = Vec<(&'ctx str, Ast<'ctx>)>;
pub type EnumField<'ctx> = (&'ctx str, Ast<'ctx>);

pub type CustomTypeField<'ctx> = Type;
pub type CustomTypeFields<'ctx> = Vec<CustomTypeField<'ctx>>;

pub type Constructor<'ctx> = Vec<(&'ctx str, Ast<'ctx>, Type, u32)>;

pub type ThrushAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
