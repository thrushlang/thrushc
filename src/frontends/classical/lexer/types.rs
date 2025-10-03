use crate::{frontends::classical::lexer::tokentype::TokenType, lazy_static};

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
        types.insert("fx86_80", TokenType::FX8680);
        types.insert("bool", TokenType::Bool);
        types.insert("char", TokenType::Char);
        types.insert("ptr", TokenType::Ptr);
        types.insert("array", TokenType::Array);
        types.insert("void", TokenType::Void);
        types.insert("Fn", TokenType::FnRef);

        types
    };
}
