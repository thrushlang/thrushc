use thrustc_attributes::ThrustAttributes;
use thrustc_span::Span;
use thrustc_typesystem::Type;

#[derive(Debug)]
pub struct Symbol {
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
    Enum {
        fields: Vec<(Type, Span)>,
        span: Span,
    },
    CustomType {
        kind: Type,
        attributes: ThrustAttributes,
        span: Span,
    },
}

impl Signature {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            Signature::Function { span, .. } => *span,
            Signature::Constant { span, .. } => *span,
            Signature::Static { span, .. } => *span,
            Signature::Struct { span, .. } => *span,
            Signature::Enum { span, .. } => *span,
            Signature::CustomType { span, .. } => *span,
        }
    }
}
