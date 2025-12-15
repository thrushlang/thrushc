use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::typesystem::types::Type;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TYPES: HashMap<&'static [u8], TokenType> = {
        let mut types: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        types.insert(b"s8", TokenType::S8);
        types.insert(b"s16", TokenType::S16);
        types.insert(b"s32", TokenType::S32);
        types.insert(b"s64", TokenType::S64);
        types.insert(b"ssize", TokenType::Ssize);
        types.insert(b"u8", TokenType::U8);
        types.insert(b"u16", TokenType::U16);
        types.insert(b"u32", TokenType::U32);
        types.insert(b"u64", TokenType::U64);
        types.insert(b"u128", TokenType::U128);
        types.insert(b"usize", TokenType::Usize);
        types.insert(b"f32", TokenType::F32);
        types.insert(b"f64", TokenType::F64);
        types.insert(b"f128", TokenType::F128);
        types.insert(b"fx86_80", TokenType::FX8680);
        types.insert(b"fppc_128", TokenType::FPPC128);
        types.insert(b"bool", TokenType::Bool);
        types.insert(b"char", TokenType::Char);
        types.insert(b"ptr", TokenType::Ptr);
        types.insert(b"array", TokenType::Array);
        types.insert(b"void", TokenType::Void);
        types.insert(b"Fn", TokenType::FnRef);

        types
    };
}
