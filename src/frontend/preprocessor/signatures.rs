use crate::frontend::{
    lexer::span::Span,
    preprocessor::types::{EnumFieldsSignature, FunctionParametersSignature},
    types::parser::stmts::types::ThrushAttributes,
    typesystem::types::Type,
};

#[derive(Debug)]
pub struct ExternalSymbol<'signature> {
    pub name: String,
    pub signature: Signature<'signature>,
    pub variant: Variant,
}

#[derive(Debug)]
pub enum Variant {
    Function,
    Constant,
    Static,

    Struct,
    Enum,
    CustomType,
}

#[derive(Debug)]
pub enum Signature<'signature> {
    Function {
        kind: Type,
        parameters: FunctionParametersSignature,
        span: Span,
        attributes: ThrushAttributes<'signature>,
    },
    Constant {
        kind: Type,
        span: Span,
        attributes: ThrushAttributes<'signature>,
    },
    Static {
        kind: Type,
        span: Span,
        attributes: ThrushAttributes<'signature>,
    },
    Struct {
        kind: Type,
        span: Span,
    },
    Enum {
        fields: EnumFieldsSignature,
        span: Span,
    },
    CustomType {
        kind: Type,
        span: Span,
    },
}
