use crate::{
    backends::classical::llvm::compiler::attributes::LLVMAttribute,
    frontends::classical::lexer::{span::Span, tokentype::TokenType},
};

impl TokenType {
    #[must_use]
    pub fn as_attribute<'ctx>(self, span: Span) -> Option<LLVMAttribute<'ctx>> {
        match self {
            TokenType::Ignore => Some(LLVMAttribute::Ignore(span)),
            TokenType::MinSize => Some(LLVMAttribute::MinSize(span)),
            TokenType::NoInline => Some(LLVMAttribute::NoInline(span)),
            TokenType::AlwaysInline => Some(LLVMAttribute::AlwaysInline(span)),
            TokenType::InlineHint => Some(LLVMAttribute::InlineHint(span)),
            TokenType::Hot => Some(LLVMAttribute::Hot(span)),
            TokenType::SafeStack => Some(LLVMAttribute::SafeStack(span)),
            TokenType::WeakStack => Some(LLVMAttribute::WeakStack(span)),
            TokenType::StrongStack => Some(LLVMAttribute::StrongStack(span)),
            TokenType::PreciseFloats => Some(LLVMAttribute::PreciseFloats(span)),
            TokenType::Stack => Some(LLVMAttribute::Stack(span)),
            TokenType::Heap => Some(LLVMAttribute::Heap(span)),
            TokenType::AsmThrow => Some(LLVMAttribute::AsmThrow(span)),
            TokenType::AsmSideEffects => Some(LLVMAttribute::AsmSideEffects(span)),
            TokenType::AsmAlignStack => Some(LLVMAttribute::AsmAlignStack(span)),
            TokenType::Packed => Some(LLVMAttribute::Packed(span)),
            TokenType::NoUnwind => Some(LLVMAttribute::NoUnwind(span)),
            TokenType::OptFuzzing => Some(LLVMAttribute::OptFuzzing(span)),

            _ => None,
        }
    }

    #[must_use]
    pub fn is_attribute(self) -> bool {
        matches!(
            self,
            TokenType::Ignore
                | TokenType::MinSize
                | TokenType::NoInline
                | TokenType::AlwaysInline
                | TokenType::InlineHint
                | TokenType::Hot
                | TokenType::SafeStack
                | TokenType::WeakStack
                | TokenType::StrongStack
                | TokenType::PreciseFloats
                | TokenType::Stack
                | TokenType::Heap
                | TokenType::AsmThrow
                | TokenType::AsmSideEffects
                | TokenType::AsmAlignStack
                | TokenType::Packed
                | TokenType::NoUnwind
                | TokenType::OptFuzzing
        )
    }
}
