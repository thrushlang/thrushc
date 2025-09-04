use crate::frontends::classical::{lexer::span::Span, types::ast::Ast, typesystem::types::Type};

impl<'ctx> Ast<'ctx> {
    #[inline]
    pub fn new_float(kind: Type, value: f64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Float {
            kind,
            value,
            signed,
            span,
        }
    }

    #[inline]
    pub fn new_integer(kind: Type, value: u64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Integer {
            kind,
            value,
            signed,
            span,
        }
    }

    #[inline]
    pub fn new_boolean(kind: Type, value: u64, span: Span) -> Ast<'ctx> {
        Ast::Boolean { kind, value, span }
    }

    #[inline]
    pub fn new_char(kind: Type, byte: u64, span: Span) -> Ast<'ctx> {
        Ast::Char { kind, byte, span }
    }

    #[inline]
    pub fn new_str(bytes: Vec<u8>, kind: Type, span: Span) -> Ast<'ctx> {
        Ast::Str { bytes, kind, span }
    }
}
