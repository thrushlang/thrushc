use crate::frontend::{
    lexer::tokentype::TokenType,
    types::{
        ast::{
            Ast,
            metadata::{
                constant::ConstantMetadata, local::LocalMetadata, staticvar::StaticMetadata,
            },
        },
        parser::stmts::types::ThrushAttributes,
    },
    typesystem::types::Type,
};

pub type BinaryOperation<'ctx> = (&'ctx Ast<'ctx>, &'ctx TokenType, &'ctx Ast<'ctx>);

pub type UnaryOperation<'ctx> = (&'ctx TokenType, &'ctx Type, &'ctx Ast<'ctx>);

pub type GlobalStatic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
    StaticMetadata,
);

pub type LocalStatic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    StaticMetadata,
);

pub type GlobalConstant<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
    ConstantMetadata,
);

pub type LocalConstant<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    ConstantMetadata,
);

pub type FunctionParameter<'ctx> = (&'ctx str, &'ctx str, &'ctx Type, u32);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
    LocalMetadata,
);

pub type GlobalFunction<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
);

pub type GlobalAssemblerFunction<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx ThrushAttributes<'ctx>,
);
