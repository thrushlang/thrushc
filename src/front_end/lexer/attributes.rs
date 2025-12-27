use crate::front_end::lexer::tokentype::TokenType;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ATTRIBUTES: HashMap<&'static [u8], TokenType> = {
        let mut attributes: HashMap<&'static [u8], TokenType> = HashMap::with_capacity(100);

        attributes.insert(b"@asmalignstack", TokenType::AsmAlignStack);
        attributes.insert(b"@asmsyntax", TokenType::AsmSyntax);
        attributes.insert(b"@asmthrow", TokenType::AsmThrow);
        attributes.insert(b"@asmeffects", TokenType::AsmSideEffects);

        attributes.insert(b"@optfuzzing", TokenType::OptFuzzing);
        attributes.insert(b"@nounwind", TokenType::NoUnwind);
        attributes.insert(b"@packed", TokenType::Packed);
        attributes.insert(b"@heap", TokenType::Heap);
        attributes.insert(b"@stack", TokenType::Stack);
        attributes.insert(b"@public", TokenType::Public);
        attributes.insert(b"@linkage", TokenType::Linkage);
        attributes.insert(b"@extern", TokenType::Extern);
        attributes.insert(b"@ignore", TokenType::Ignore);
        attributes.insert(b"@hot", TokenType::Hot);
        attributes.insert(b"@minsize", TokenType::MinSize);
        attributes.insert(b"@alwaysinline", TokenType::AlwaysInline);
        attributes.insert(b"@noinline", TokenType::NoInline);
        attributes.insert(b"@inline", TokenType::InlineHint);
        attributes.insert(b"@safestack", TokenType::SafeStack);
        attributes.insert(b"@weakstack", TokenType::WeakStack);
        attributes.insert(b"@strongstack", TokenType::StrongStack);
        attributes.insert(b"@precisefp", TokenType::PreciseFloats);
        attributes.insert(b"@convention", TokenType::Convention);
        attributes.insert(b"@constructor", TokenType::Constructor);
        attributes.insert(b"@destructor", TokenType::Destructor);

        attributes
    };
}
