use crate::frontend::{
    lexer::span::Span,
    types::{ast::Ast, lexer::ThrushType},
};

impl<'ctx> Ast<'ctx> {
    pub fn new_float(kind: ThrushType, value: f64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Float {
            kind,
            value,
            signed,
            span,
        }
    }

    pub fn new_integer(kind: ThrushType, value: u64, signed: bool, span: Span) -> Ast<'ctx> {
        Ast::Integer {
            kind,
            value,
            signed,
            span,
        }
    }

    pub fn new_boolean(kind: ThrushType, value: u64, span: Span) -> Ast<'ctx> {
        Ast::Boolean { kind, value, span }
    }

    pub fn new_char(kind: ThrushType, byte: u64, span: Span) -> Ast<'ctx> {
        Ast::Char { kind, byte, span }
    }

    pub fn new_str(bytes: Vec<u8>, kind: ThrushType, span: Span) -> Ast<'ctx> {
        Ast::Str { bytes, kind, span }
    }
}
