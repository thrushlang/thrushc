use crate::front_end::lexer::tokentype::TokenType;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTINS: HashMap<&'static str, TokenType> = {
        let mut builtins: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        builtins.insert("halloc", TokenType::Halloc);
        builtins.insert("sizeof", TokenType::SizeOf);
        builtins.insert("memset", TokenType::MemSet);
        builtins.insert("memmove", TokenType::MemMove);
        builtins.insert("memcpy", TokenType::MemCpy);
        builtins.insert("alignof", TokenType::AlignOf);

        builtins
    };
}
