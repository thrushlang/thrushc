use crate::frontend::{
    lexer::tokentype::TokenType,
    types::{
        lexer::ThrushType,
        parser::stmts::{stmt::ThrushStatement, types::ThrushAttributes},
    },
};

pub type BinaryOperation<'ctx> = (
    &'ctx ThrushStatement<'ctx>,
    &'ctx TokenType,
    &'ctx ThrushStatement<'ctx>,
);

pub type UnaryOperation<'ctx> = (
    &'ctx TokenType,
    &'ctx ThrushType,
    &'ctx ThrushStatement<'ctx>,
);

pub type FunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [ThrushStatement<'ctx>],
    &'ctx [ThrushType],
    &'ctx ThrushStatement<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type AssemblerFunctionRepresentation<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx [ThrushStatement<'ctx>],
    &'ctx [ThrushType],
    &'ctx ThrushAttributes<'ctx>,
);

pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx str, &'ctx ThrushType, u32, bool);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx ThrushType,
    &'ctx ThrushStatement<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);
