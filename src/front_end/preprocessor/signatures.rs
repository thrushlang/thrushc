use crate::front_end::{
    lexer::span::Span,
    preprocessor::types::{EnumFieldsSignature, FunctionParametersSignature},
    types::parser::stmts::types::ThrushAttributes,
    typesystem::types::Type,
};

#[derive(Debug)]
pub struct ExternalSymbol {
    pub name: String,
    pub signature: Signature,
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
pub enum Signature {
    Function {
        kind: Type,
        parameters: FunctionParametersSignature,
        span: Span,
        attributes: ThrushAttributes,
    },
    Constant {
        kind: Type,
        span: Span,
        attributes: ThrushAttributes,
    },
    Static {
        kind: Type,
        span: Span,
        attributes: ThrushAttributes,
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
