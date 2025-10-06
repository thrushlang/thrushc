use crate::{frontends::classical::lexer::tokentype::TokenType, lazy_static};

use ahash::AHashMap as HashMap;

lazy_static! {
    pub static ref ATTRIBUTES: HashMap<&'static str, TokenType> = {
        let mut attributes: HashMap<&'static str, TokenType> = HashMap::with_capacity(100);

        attributes.insert("@asmalignstack", TokenType::AsmAlignStack);
        attributes.insert("@asmsyntax", TokenType::AsmSyntax);
        attributes.insert("@asmthrow", TokenType::AsmThrow);
        attributes.insert("@asmeffects", TokenType::AsmSideEffects);

        attributes.insert("@optfuzzing", TokenType::OptFuzzing);
        attributes.insert("@nounwind", TokenType::NoUnwind);
        attributes.insert("@packed", TokenType::Packed);
        attributes.insert("@heap", TokenType::Heap);
        attributes.insert("@stack", TokenType::Stack);
        attributes.insert("@public", TokenType::Public);
        attributes.insert("@extern", TokenType::Extern);
        attributes.insert("@ignore", TokenType::Ignore);
        attributes.insert("@hot", TokenType::Hot);
        attributes.insert("@minsize", TokenType::MinSize);
        attributes.insert("@alwaysinline", TokenType::AlwaysInline);
        attributes.insert("@noinline", TokenType::NoInline);
        attributes.insert("@inline", TokenType::InlineHint);
        attributes.insert("@safestack", TokenType::SafeStack);
        attributes.insert("@weakstack", TokenType::WeakStack);
        attributes.insert("@strongstack", TokenType::StrongStack);
        attributes.insert("@precisefp", TokenType::PreciseFloats);
        attributes.insert("@convention", TokenType::Convention);

        attributes
    };
}
