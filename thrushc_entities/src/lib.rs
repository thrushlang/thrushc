use thrushc_ast::{
    Ast,
    metadata::{ConstantMetadata, FunctionParameterMetadata, LocalMetadata, StaticMetadata},
};
use thrushc_attributes::ThrushAttributes;
use thrushc_span::Span;
use thrushc_token::tokentype::TokenType;
use thrushc_typesystem::Type;

pub mod analyzer;
pub mod linter;
pub mod typechecker;

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

pub type LocalVariable<'ctx> = (
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

pub fn assembler_function_from_ast<'entity>(ast: &'entity Ast) -> AssemblerFunction<'entity> {
    if let Ast::AssemblerFunction {
        name,
        ascii_name,
        assembler,
        constraints,
        parameters_types,
        parameters,
        return_type,
        attributes,
        span,
        ..
    } = ast
    {
        return (
            name,
            ascii_name,
            assembler,
            constraints,
            return_type,
            parameters,
            parameters_types,
            attributes,
            *span,
        );
    }

    unreachable!()
}

pub fn intrinsic_from_ast<'entity>(ast: &'entity Ast) -> Intrinsic<'entity> {
    if let Ast::Intrinsic {
        name,
        external_name,
        parameters,
        parameters_types,
        return_type,
        attributes,
        span,
        ..
    } = ast
    {
        return (
            name,
            external_name,
            return_type,
            parameters,
            parameters_types,
            attributes,
            *span,
        );
    }

    unreachable!()
}

pub fn function_from_ast<'entity>(ast: &'entity Ast) -> Function<'entity> {
    if let Ast::Function {
        name,
        ascii_name,
        parameters,
        parameter_types,
        body,
        return_type,
        attributes,
        span,
        ..
    } = ast
    {
        return (
            name,
            ascii_name,
            return_type,
            parameters,
            parameter_types,
            body.as_deref(),
            attributes,
            *span,
        );
    }

    unreachable!()
}

pub fn global_static_from_ast<'entity>(ast: &'entity Ast) -> GlobalStatic<'entity> {
    if let Ast::Static {
        name,
        ascii_name,
        kind,
        value,
        attributes,
        metadata,
        span,
        ..
    } = ast
    {
        return (
            name,
            ascii_name,
            kind,
            value.as_deref(),
            attributes,
            *metadata,
            *span,
        );
    }

    unreachable!()
}

pub fn local_static_from_ast<'entity>(ast: &'entity Ast) -> LocalStatic<'entity> {
    if let Ast::Static {
        name,
        ascii_name,
        kind,
        value,
        metadata,
        span,
        ..
    } = ast
    {
        return (name, ascii_name, kind, value.as_deref(), *metadata, *span);
    }

    unreachable!()
}

pub fn global_constant_from_ast<'entity>(ast: &'entity Ast) -> GlobalConstant<'entity> {
    if let Ast::Const {
        name,
        ascii_name,
        kind,
        value,
        attributes,
        metadata,
        span,
        ..
    } = ast
    {
        return (
            name, ascii_name, kind, &**value, attributes, *metadata, *span,
        );
    }

    unreachable!()
}

pub fn local_constant_from_ast<'entity>(ast: &'entity Ast) -> LocalConstant<'entity> {
    if let Ast::Const {
        name,
        ascii_name,
        kind,
        value,
        metadata,
        span,
        ..
    } = ast
    {
        return (name, ascii_name, kind, &**value, *metadata, *span);
    }

    unreachable!()
}

pub fn local_variable_from_ast<'entity>(ast: &'entity Ast) -> LocalVariable<'entity> {
    if let Ast::Local {
        name,
        ascii_name,
        kind,
        value,
        attributes,
        metadata,
        span,
        ..
    } = ast
    {
        return (
            name,
            ascii_name,
            kind,
            value.as_deref(),
            attributes,
            *metadata,
            *span,
        );
    }

    unreachable!()
}

#[inline]
pub fn function_parameter_from_ast<'entity>(ast: &'entity Ast) -> FunctionParameter<'entity> {
    if let Ast::FunctionParameter {
        name,
        ascii_name,
        kind,
        position,
        metadata,
        span,
    } = ast
    {
        return (name, ascii_name, kind, *position, *span, *metadata);
    }

    unreachable!()
}
