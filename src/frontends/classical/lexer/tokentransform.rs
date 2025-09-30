use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        typesystem::types::Type,
    },
};

impl TokenType {
    pub fn as_type(&self, span: Span) -> Result<Type, ThrushCompilerIssue> {
        match self {
            TokenType::Char => Ok(Type::Char),

            TokenType::S8 => Ok(Type::S8),
            TokenType::S16 => Ok(Type::S16),
            TokenType::S32 => Ok(Type::S32),
            TokenType::S64 => Ok(Type::S64),

            TokenType::U8 => Ok(Type::U8),
            TokenType::U16 => Ok(Type::U16),
            TokenType::U32 => Ok(Type::U32),
            TokenType::U64 => Ok(Type::U64),

            TokenType::Bool => Ok(Type::Bool),

            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),

            TokenType::Ptr => Ok(Type::Ptr(None)),
            TokenType::Addr => Ok(Type::Addr),
            TokenType::Void => Ok(Type::Void),

            any => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                format!("{} isn't a valid type.", any),
                None,
                span,
            )),
        }
    }
}
