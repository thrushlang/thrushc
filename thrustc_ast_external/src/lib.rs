use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_typesystem::Type;

#[derive(Debug, Clone)]
pub struct ExternalSymbol {
    pub name: String,
    pub signature: ExternalSignature,
    pub variant: ExternalVariant,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExternalVariant {
    Function,
    Constant,
    Static,

    Struct,
    Enum,
    CustomType,
}

#[derive(Debug, Clone)]
pub enum ExternalSignature {
    Function {
        kind: Type,
        parameters: Vec<(Type, Span)>,
        attributes: ThrustAttributes,
        span: Span,
    },
    Constant {
        kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
    Static {
        kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
    Struct {
        kind: Type,
        span: Span,
    },
    CustomType {
        kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
}
