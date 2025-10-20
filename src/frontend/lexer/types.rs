use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        typesystem::types::Type,
    },
    lazy_static,
};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref TYPES: HashMap<&'static str, TokenType> = {
        let mut types: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        types.insert("s8", TokenType::S8);
        types.insert("s16", TokenType::S16);
        types.insert("s32", TokenType::S32);
        types.insert("s64", TokenType::S64);
        types.insert("u8", TokenType::U8);
        types.insert("u16", TokenType::U16);
        types.insert("u32", TokenType::U32);
        types.insert("u64", TokenType::U64);
        types.insert("u128", TokenType::U128);
        types.insert("f32", TokenType::F32);
        types.insert("f64", TokenType::F64);
        types.insert("f128", TokenType::F128);
        types.insert("fx86_80", TokenType::FX8680);
        types.insert("fppc_128", TokenType::FPPC128);
        types.insert("bool", TokenType::Bool);
        types.insert("char", TokenType::Char);
        types.insert("ptr", TokenType::Ptr);
        types.insert("array", TokenType::Array);
        types.insert("void", TokenType::Void);
        types.insert("Fn", TokenType::FnRef);

        types
    };
}

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
            TokenType::U128 => Ok(Type::U128),

            TokenType::Bool => Ok(Type::Bool),

            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),
            TokenType::F128 => Ok(Type::F128),

            TokenType::FX8680 => Ok(Type::FX8680),
            TokenType::FPPC128 => Ok(Type::FPPC128),

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
