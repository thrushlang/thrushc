use crate::frontend::{
    lexer::tokentype::TokenType,
    types::{ast::Ast, parser::stmts::types::ThrushAttributes},
    typesystem::types::Type,
};

pub type BinaryOperation<'ctx> = (&'ctx Ast<'ctx>, &'ctx TokenType, &'ctx Ast<'ctx>);

pub type UnaryOperation<'ctx> = (&'ctx TokenType, &'ctx Type, &'ctx Ast<'ctx>);

pub type StaticRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type ConstantRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type FunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type AssemblerFunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx ThrushAttributes<'ctx>,
);

pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx str, &'ctx Type, u32);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);
