use crate::frontends::classical::{
    lexer::{span::Span, tokentype::TokenType},
    types::{
        ast::{
            Ast,
            metadata::{
                constant::ConstantMetadata, fnparam::FunctionParameterMetadata,
                local::LocalMetadata, staticvar::StaticMetadata,
            },
        },
        parser::stmts::types::ThrushAttributes,
    },
    typesystem::types::Type,
};

pub type BinaryOperation<'ctx> = (&'ctx Ast<'ctx>, &'ctx TokenType, &'ctx Ast<'ctx>, Span);

pub type UnaryOperation<'ctx> = (&'ctx TokenType, &'ctx Type, &'ctx Ast<'ctx>);

pub type GlobalStatic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    Option<&'ctx Ast<'ctx>>,
    &'ctx ThrushAttributes<'ctx>,
    StaticMetadata,
    Span,
);

pub type LocalStatic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    Option<&'ctx Ast<'ctx>>,
    StaticMetadata,
    Span,
);

pub type GlobalConstant<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    &'ctx ThrushAttributes<'ctx>,
    ConstantMetadata,
    Span,
);

pub type LocalConstant<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx Ast<'ctx>,
    ConstantMetadata,
    Span,
);

pub type FunctionParameter<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    u32,
    Span,
    FunctionParameterMetadata,
);

pub type Local<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    Option<&'ctx Ast<'ctx>>,
    &'ctx ThrushAttributes<'ctx>,
    LocalMetadata,
    Span,
);

pub type GlobalFunction<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    Option<&'ctx Ast<'ctx>>,
    &'ctx ThrushAttributes<'ctx>,
    Span,
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
    Span,
);
