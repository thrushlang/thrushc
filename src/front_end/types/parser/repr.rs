use crate::core::diagnostic::span::Span;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::constant::ConstantMetadata;
use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::ast::metadata::local::LocalMetadata;
use crate::front_end::types::ast::metadata::staticvar::StaticMetadata;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub type BinaryOperation<'ctx> = (&'ctx Ast<'ctx>, &'ctx TokenType, &'ctx Ast<'ctx>, Span);

pub type UnaryOperation<'ctx> = (&'ctx TokenType, &'ctx Type, &'ctx Ast<'ctx>);

pub type GlobalStatic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    Option<&'ctx Ast<'ctx>>,
    &'ctx ThrushAttributes,
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
    &'ctx ThrushAttributes,
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
    &'ctx ThrushAttributes,
    LocalMetadata,
    Span,
);

pub type Function<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    Option<&'ctx Ast<'ctx>>,
    &'ctx ThrushAttributes,
    Span,
);

pub type AssemblerFunction<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx ThrushAttributes,
    Span,
);

pub type Intrinsic<'ctx> = (
    &'ctx str,
    &'ctx str,
    &'ctx Type,
    &'ctx [Ast<'ctx>],
    &'ctx [Type],
    &'ctx ThrushAttributes,
    Span,
);
