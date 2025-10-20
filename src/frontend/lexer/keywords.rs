use crate::{frontend::lexer::tokentype::TokenType, lazy_static};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut keywords: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        keywords.insert("local", TokenType::Local);
        keywords.insert("fn", TokenType::Fn);
        keywords.insert("if", TokenType::If);
        keywords.insert("elif", TokenType::Elif);
        keywords.insert("else", TokenType::Else);
        keywords.insert("for", TokenType::For);
        keywords.insert("while", TokenType::While);
        keywords.insert("loop", TokenType::Loop);
        keywords.insert("true", TokenType::True);
        keywords.insert("false", TokenType::False);
        keywords.insert("or", TokenType::Or);
        keywords.insert("and", TokenType::And);
        keywords.insert("const", TokenType::Const);
        keywords.insert("struct", TokenType::Struct);
        keywords.insert("return", TokenType::Return);
        keywords.insert("break", TokenType::Break);
        keywords.insert("continue", TokenType::Continue);
        keywords.insert("pass", TokenType::Pass);
        keywords.insert("instr", TokenType::Instr);
        keywords.insert("mut", TokenType::Mut);
        keywords.insert("nullptr", TokenType::NullPtr);
        keywords.insert("as", TokenType::As);
        keywords.insert("asmfn", TokenType::AsmFn);
        keywords.insert("asm", TokenType::Asm);
        keywords.insert("global_asm", TokenType::GlobalAsm);
        keywords.insert("defer", TokenType::Defer);
        keywords.insert("type", TokenType::Type);
        keywords.insert("enum", TokenType::Enum);
        keywords.insert("alloc", TokenType::Alloc);
        keywords.insert("address", TokenType::Address);
        keywords.insert("addr", TokenType::Addr);
        keywords.insert("load", TokenType::Load);
        keywords.insert("write", TokenType::Write);
        keywords.insert("fixed", TokenType::Fixed);
        keywords.insert("ref", TokenType::DirectRef);
        keywords.insert("static", TokenType::Static);
        keywords.insert("indirect", TokenType::Indirect);
        keywords.insert("unreachable", TokenType::Unreachable);
        keywords.insert("import", TokenType::Import);
        keywords.insert("new", TokenType::New);

        keywords
    };
}
