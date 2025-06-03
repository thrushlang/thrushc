use crate::{lazy_static, types::frontend::lexer::tokenkind::TokenKind};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref THRUSH_KEYWORDS: HashMap<&'static [u8], TokenKind> = {
        let mut keywords: HashMap<&'static [u8], TokenKind> = HashMap::with_capacity(100);

        keywords.insert(b"local", TokenKind::Local);
        keywords.insert(b"fn", TokenKind::Fn);
        keywords.insert(b"if", TokenKind::If);
        keywords.insert(b"elif", TokenKind::Elif);
        keywords.insert(b"else", TokenKind::Else);
        keywords.insert(b"for", TokenKind::For);
        keywords.insert(b"while", TokenKind::While);
        keywords.insert(b"loop", TokenKind::Loop);
        keywords.insert(b"true", TokenKind::True);
        keywords.insert(b"false", TokenKind::False);
        keywords.insert(b"or", TokenKind::Or);
        keywords.insert(b"and", TokenKind::And);
        keywords.insert(b"const", TokenKind::Const);
        keywords.insert(b"struct", TokenKind::Struct);
        keywords.insert(b"return", TokenKind::Return);
        keywords.insert(b"break", TokenKind::Break);
        keywords.insert(b"continue", TokenKind::Continue);
        keywords.insert(b"methods", TokenKind::Methods);
        keywords.insert(b"this", TokenKind::This);
        keywords.insert(b"pass", TokenKind::Pass);
        keywords.insert(b"match", TokenKind::Match);
        keywords.insert(b"pattern", TokenKind::Pattern);
        keywords.insert(b"instr", TokenKind::Instr);
        keywords.insert(b"mut", TokenKind::Mut);
        keywords.insert(b"nullptr", TokenKind::NullPtr);
        keywords.insert(b"castptr", TokenKind::CastPtr);
        keywords.insert(b"transmute", TokenKind::Transmute);
        keywords.insert(b"type", TokenKind::Type);
        keywords.insert(b"enum", TokenKind::Enum);
        keywords.insert(b"alloc", TokenKind::Alloc);
        keywords.insert(b"heap!", TokenKind::Heap);
        keywords.insert(b"stack!", TokenKind::Stack);
        keywords.insert(b"static!", TokenKind::Static);
        keywords.insert(b"address", TokenKind::Address);
        keywords.insert(b"addr", TokenKind::Addr);
        keywords.insert(b"load", TokenKind::Load);
        keywords.insert(b"write", TokenKind::Write);
        keywords.insert(b"@import", TokenKind::Import);
        keywords.insert(b"@public", TokenKind::Public);
        keywords.insert(b"@extern", TokenKind::Extern);
        keywords.insert(b"@ignore", TokenKind::Ignore);
        keywords.insert(b"@hot", TokenKind::Hot);
        keywords.insert(b"@minsize", TokenKind::MinSize);
        keywords.insert(b"@alwaysinline", TokenKind::AlwaysInline);
        keywords.insert(b"@noinline", TokenKind::NoInline);
        keywords.insert(b"@inline", TokenKind::InlineHint);
        keywords.insert(b"@safestack", TokenKind::SafeStack);
        keywords.insert(b"@weakstack", TokenKind::WeakStack);
        keywords.insert(b"@strongstack", TokenKind::StrongStack);
        keywords.insert(b"@precisefp", TokenKind::PreciseFloats);
        keywords.insert(b"@convention", TokenKind::Convention);
        keywords.insert(b"new", TokenKind::New);

        keywords.insert(b"s8", TokenKind::S8);
        keywords.insert(b"s16", TokenKind::S16);
        keywords.insert(b"s32", TokenKind::S32);
        keywords.insert(b"s64", TokenKind::S64);
        keywords.insert(b"u8", TokenKind::U8);
        keywords.insert(b"u16", TokenKind::U16);
        keywords.insert(b"u32", TokenKind::U32);
        keywords.insert(b"u64", TokenKind::U64);
        keywords.insert(b"f32", TokenKind::F32);
        keywords.insert(b"f64", TokenKind::F64);
        keywords.insert(b"bool", TokenKind::Bool);
        keywords.insert(b"char", TokenKind::Char);
        keywords.insert(b"ptr", TokenKind::Ptr);
        keywords.insert(b"str", TokenKind::Str);
        keywords.insert(b"void", TokenKind::Void);

        keywords
    };
}
