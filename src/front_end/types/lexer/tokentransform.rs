use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::lexer::traits::TokenTypeTypeTransform;
use crate::front_end::typesystem::types::Type;

impl TokenTypeTypeTransform for TokenType {
    fn as_type(&self, span: Span) -> Result<Type, CompilationIssue> {
        match self {
            TokenType::Char => Ok(Type::Char(span)),

            TokenType::S8 => Ok(Type::S8(span)),
            TokenType::S16 => Ok(Type::S16(span)),
            TokenType::S32 => Ok(Type::S32(span)),
            TokenType::S64 => Ok(Type::S64(span)),
            TokenType::Ssize => Ok(Type::SSize(span)),

            TokenType::U8 => Ok(Type::U8(span)),
            TokenType::U16 => Ok(Type::U16(span)),
            TokenType::U32 => Ok(Type::U32(span)),
            TokenType::U64 => Ok(Type::U64(span)),
            TokenType::U128 => Ok(Type::U128(span)),
            TokenType::Usize => Ok(Type::USize(span)),

            TokenType::Bool => Ok(Type::Bool(span)),

            TokenType::F32 => Ok(Type::F32(span)),
            TokenType::F64 => Ok(Type::F64(span)),
            TokenType::F128 => Ok(Type::F128(span)),

            TokenType::FX8680 => Ok(Type::FX8680(span)),
            TokenType::FPPC128 => Ok(Type::FPPC128(span)),

            TokenType::Ptr => Ok(Type::Ptr(None, span)),
            TokenType::Addr => Ok(Type::Addr(span)),
            TokenType::Void => Ok(Type::Void(span)),

            any => Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                format!("{} isn't a valid type.", any),
                None,
                span,
            )),
        }
    }

    fn as_type_preprocessor(&self, span: Span) -> Result<Type, ()> {
        match self {
            TokenType::Char => Ok(Type::Char(span)),

            TokenType::S8 => Ok(Type::S8(span)),
            TokenType::S16 => Ok(Type::S16(span)),
            TokenType::S32 => Ok(Type::S32(span)),
            TokenType::S64 => Ok(Type::S64(span)),
            TokenType::Ssize => Ok(Type::SSize(span)),

            TokenType::U8 => Ok(Type::U8(span)),
            TokenType::U16 => Ok(Type::U16(span)),
            TokenType::U32 => Ok(Type::U32(span)),
            TokenType::U64 => Ok(Type::U64(span)),
            TokenType::U128 => Ok(Type::U128(span)),
            TokenType::Usize => Ok(Type::USize(span)),

            TokenType::Bool => Ok(Type::Bool(span)),

            TokenType::F32 => Ok(Type::F32(span)),
            TokenType::F64 => Ok(Type::F64(span)),
            TokenType::F128 => Ok(Type::F128(span)),

            TokenType::FX8680 => Ok(Type::FX8680(span)),
            TokenType::FPPC128 => Ok(Type::FPPC128(span)),

            TokenType::Ptr => Ok(Type::Ptr(None, span)),
            TokenType::Addr => Ok(Type::Addr(span)),
            TokenType::Void => Ok(Type::Void(span)),

            _ => Err(()),
        }
    }
}
