use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_typesystem::Type;

#[derive(Debug, Clone)]
pub struct ExternalSymbol {
    pub name: String,
    pub signature: ExternalSignature,
    pub variant: ExternalVariant,
}

impl ExternalSymbol {
    #[inline]
    pub fn new(name: String, signature: ExternalSignature, variant: ExternalVariant) -> Self {
        Self {
            name,
            signature,
            variant,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExternalVariant {
    Function,
    Constant,
    Static,
    Struct,
    CustomType,
    Unavailable,
}

#[derive(Debug, Clone)]
pub enum ExternalSignature {
    Function {
        kind: Type,
        invalid_kind: Type,
        parameters: Vec<(Type, Span)>,
        attributes: ThrustAttributes,
        span: Span,
    },
    Constant {
        kind: Type,
        invalid_kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
    Static {
        kind: Type,
        invalid_kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
    Struct {
        kind: Type,
        invalid_kind: Type,
        span: Span,
    },
    CustomType {
        kind: Type,
        invalid_kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
    Unavailable {
        kind: Type,
        span: Span,
    },
}
