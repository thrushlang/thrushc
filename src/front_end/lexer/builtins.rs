use crate::front_end::lexer::tokentype::TokenType;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTINS: HashMap<&'static [u8], TokenType> = {
        let mut builtins: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        builtins.insert(b"halloc", TokenType::Halloc);
        builtins.insert(b"size_of", TokenType::SizeOf);
        builtins.insert(b"memset", TokenType::MemSet);
        builtins.insert(b"memmove", TokenType::MemMove);
        builtins.insert(b"memcpy", TokenType::MemCpy);
        builtins.insert(b"align_of", TokenType::AlignOf);
        builtins.insert(b"abi_size_of", TokenType::AbiSizeOf);
        builtins.insert(b"bit_size_of", TokenType::BitSizeOf);
        builtins.insert(b"abi_align_of", TokenType::AbiAlignOf);

        builtins
    };
}
