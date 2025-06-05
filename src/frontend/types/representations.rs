use crate::frontend::{
    lexer::tokenkind::TokenKind,
    types::{
        lexer::ThrushType,
        parser::stmts::{stmt::ThrushStatement, types::ThrushAttributes},
    },
};

pub type BinaryOperation<'ctx> = (
    &'ctx ThrushStatement<'ctx>,
    &'ctx TokenKind,
    &'ctx ThrushStatement<'ctx>,
);

pub type UnaryOperation<'ctx> = (
    &'ctx TokenKind,
    &'ctx ThrushType,
    &'ctx ThrushStatement<'ctx>,
);

pub type FunctionRepresentation<'ctx> = (
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
    &'ctx ThrushType,
    &'ctx [ThrushStatement<'ctx>],
    &'ctx [ThrushType],
    &'ctx ThrushAttributes<'ctx>,
);

pub type FunctionCall<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx [ThrushStatement<'ctx>]);

pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx ThrushType, u32, bool);

pub type Local<'ctx> = (&'ctx str, &'ctx ThrushType, &'ctx ThrushStatement<'ctx>);
