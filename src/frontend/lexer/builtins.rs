use crate::{frontend::lexer::tokentype::TokenType, lazy_static};

use ahash::AHashMap as HashMap;

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
