use crate::front_end::lexer::tokentype::TokenType;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static [u8], TokenType> = {
        let mut keywords: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        keywords.insert(b"local", TokenType::Local);
        keywords.insert(b"fn", TokenType::Fn);
        keywords.insert(b"if", TokenType::If);
        keywords.insert(b"elif", TokenType::Elif);
        keywords.insert(b"else", TokenType::Else);
        keywords.insert(b"for", TokenType::For);
        keywords.insert(b"while", TokenType::While);
        keywords.insert(b"loop", TokenType::Loop);
        keywords.insert(b"true", TokenType::True);
        keywords.insert(b"false", TokenType::False);
        keywords.insert(b"or", TokenType::Or);
        keywords.insert(b"and", TokenType::And);
        keywords.insert(b"const", TokenType::Const);
        keywords.insert(b"struct", TokenType::Struct);
        keywords.insert(b"return", TokenType::Return);
        keywords.insert(b"break", TokenType::Break);
        keywords.insert(b"continue", TokenType::Continue);
        keywords.insert(b"pass", TokenType::Pass);
        keywords.insert(b"instr", TokenType::Instr);
        keywords.insert(b"mut", TokenType::Mut);
        keywords.insert(b"nullptr", TokenType::NullPtr);
        keywords.insert(b"as", TokenType::As);
        keywords.insert(b"asmfn", TokenType::AsmFn);
        keywords.insert(b"asm", TokenType::Asm);
        keywords.insert(b"global_asm", TokenType::GlobalAsm);
        keywords.insert(b"deref", TokenType::Deref);
        keywords.insert(b"type", TokenType::Type);
        keywords.insert(b"enum", TokenType::Enum);
        keywords.insert(b"alloc", TokenType::Alloc);
        keywords.insert(b"address", TokenType::Address);
        keywords.insert(b"addr", TokenType::Addr);
        keywords.insert(b"load", TokenType::Load);
        keywords.insert(b"write", TokenType::Write);
        keywords.insert(b"fixed", TokenType::Fixed);
        keywords.insert(b"ref", TokenType::DirectRef);
        keywords.insert(b"static", TokenType::Static);
        keywords.insert(b"indirect", TokenType::Indirect);
        keywords.insert(b"unreachable", TokenType::Unreachable);
        keywords.insert(b"intrinsic", TokenType::Intrinsic);
        keywords.insert(b"import", TokenType::Import);
        keywords.insert(b"importC", TokenType::ImportC);
        keywords.insert(b"new", TokenType::New);

        keywords
    };
}
