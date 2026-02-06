use thrushc_attributes::ThrushAttributes;
use thrushc_span::Span;
use thrushc_typesystem::Type;

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
        attributes: ThrushAttributes,
        span: Span,
    },
    Constant {
        kind: Type,
        attributes: ThrushAttributes,
        span: Span,
    },
    Static {
        kind: Type,
        attributes: ThrushAttributes,
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
        attributes: ThrushAttributes,
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
