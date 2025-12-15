use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::lexer::traits::TokenTypeTypeTransform;
use crate::front_end::typesystem::types::Type;

impl TokenTypeTypeTransform for TokenType {
    fn as_type(&self, span: Span) -> Result<Type, CompilationIssue> {
        match self {
            TokenType::Char => Ok(Type::Char),

            TokenType::S8 => Ok(Type::S8),
            TokenType::S16 => Ok(Type::S16),
            TokenType::S32 => Ok(Type::S32),
            TokenType::S64 => Ok(Type::S64),
            TokenType::Ssize => Ok(Type::SSize),

            TokenType::U8 => Ok(Type::U8),
            TokenType::U16 => Ok(Type::U16),
            TokenType::U32 => Ok(Type::U32),
            TokenType::U64 => Ok(Type::U64),
            TokenType::U128 => Ok(Type::U128),
            TokenType::Usize => Ok(Type::USize),

            TokenType::Bool => Ok(Type::Bool),

            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),
            TokenType::F128 => Ok(Type::F128),

            TokenType::FX8680 => Ok(Type::FX8680),
            TokenType::FPPC128 => Ok(Type::FPPC128),

            TokenType::Ptr => Ok(Type::Ptr(None)),
            TokenType::Addr => Ok(Type::Addr),
            TokenType::Void => Ok(Type::Void),

            any => Err(CompilationIssue::Error(
                "Syntax error".into(),
                format!("{} isn't a valid type.", any),
                None,
                span,
            )),
        }
    }

    fn as_type_preprocessor(&self) -> Result<Type, ()> {
        match self {
            TokenType::Char => Ok(Type::Char),

            TokenType::S8 => Ok(Type::S8),
            TokenType::S16 => Ok(Type::S16),
            TokenType::S32 => Ok(Type::S32),
            TokenType::S64 => Ok(Type::S64),
            TokenType::Ssize => Ok(Type::SSize),

            TokenType::U8 => Ok(Type::U8),
            TokenType::U16 => Ok(Type::U16),
            TokenType::U32 => Ok(Type::U32),
            TokenType::U64 => Ok(Type::U64),
            TokenType::U128 => Ok(Type::U128),
            TokenType::Usize => Ok(Type::USize),

            TokenType::Bool => Ok(Type::Bool),

            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),
            TokenType::F128 => Ok(Type::F128),

            TokenType::FX8680 => Ok(Type::FX8680),
            TokenType::FPPC128 => Ok(Type::FPPC128),

            TokenType::Ptr => Ok(Type::Ptr(None)),
            TokenType::Addr => Ok(Type::Addr),
            TokenType::Void => Ok(Type::Void),

            _ => Err(()),
        }
    }
}
