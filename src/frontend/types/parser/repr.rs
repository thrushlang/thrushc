use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::constant::ConstantMetadata;
use crate::frontend::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::frontend::types::ast::metadata::local::LocalMetadata;
use crate::frontend::types::ast::metadata::staticvar::StaticMetadata;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::types::Type;

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
