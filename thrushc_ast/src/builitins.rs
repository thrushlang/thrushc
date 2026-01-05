use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::Ast;

#[derive(Debug, Clone)]
pub enum ThrushBuiltin<'mir> {
    Halloc {
        of: Type,
        span: Span,
    },
    MemCpy {
        src: std::boxed::Box<Ast<'mir>>,
        dst: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    MemMove {
        src: std::boxed::Box<Ast<'mir>>,
        dst: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    MemSet {
        dst: std::boxed::Box<Ast<'mir>>,
        new_size: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    BitSizeOf {
        of: Type,
        span: Span,
    },
    AbiSizeOf {
        of: Type,
        span: Span,
    },
    AbiAlignOf {
        of: Type,
        span: Span,
    },
    AlignOf {
        of: Type,
        span: Span,
    },
    SizeOf {
        of: Type,
        span: Span,
    },
}
