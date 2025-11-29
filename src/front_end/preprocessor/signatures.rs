use crate::{
    core::diagnostic::span::Span,
    front_end::{
        preprocessor::types::{EnumFieldsSignature, FunctionParametersSignature},
        typesystem::types::Type,
    },
    middle_end::mir::attributes::ThrushAttributes,
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
