use crate::frontend::{
    lexer::tokentype::TokenType,
    types::{ast::Ast, lexer::ThrushType, parser::stmts::types::ThrushAttributes},
};

pub type BinaryOperation<'ctx> = (&'ctx Ast<'ctx>, &'ctx TokenType, &'ctx Ast<'ctx>);

pub type UnaryOperation<'ctx> = (&'ctx TokenType, &'ctx ThrushType, &'ctx Ast<'ctx>);

pub type FunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [Ast<'ctx>],
    &'ctx [ThrushType],
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type AssemblerFunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [Ast<'ctx>],
    &'ctx [ThrushType],
    &'ctx ThrushAttributes<'ctx>,
);

pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx str, &'ctx ThrushType, u32, bool);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);
