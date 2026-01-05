use thrushc_span::Span;

use crate::{Type, traits::TypeCodeLocation};

impl TypeCodeLocation for Type {
    fn get_span(&self) -> Span {
        match self {
            Type::Char(span)
            | Type::S8(span)
            | Type::S16(span)
            | Type::S32(span)
            | Type::S64(span)
            | Type::SSize(span)
            | Type::U8(span)
            | Type::U16(span)
            | Type::U32(span)
            | Type::U64(span)
            | Type::U128(span)
            | Type::USize(span)
            | Type::F32(span)
            | Type::F64(span)
            | Type::F128(span)
            | Type::FX8680(span)
            | Type::FPPC128(span)
            | Type::Bool(span)
            | Type::Void(span)
            | Type::Addr(span)
            | Type::Array(_, span)
            | Type::FixedArray(_, _, span)
            | Type::Const(_, span)
            | Type::Ptr(_, span)
            | Type::Struct(_, _, _, span)
            | Type::Fn(_, _, _, span) => *span,
        }
    }
}
