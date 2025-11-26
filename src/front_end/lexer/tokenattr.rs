use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::attributes::ThrushAttribute;

impl TokenType {
    #[must_use]
    pub fn as_attribute(self, span: Span) -> Option<ThrushAttribute> {
        match self {
            TokenType::Ignore => Some(ThrushAttribute::Ignore(span)),
            TokenType::MinSize => Some(ThrushAttribute::MinSize(span)),
            TokenType::NoInline => Some(ThrushAttribute::NoInline(span)),
            TokenType::AlwaysInline => Some(ThrushAttribute::AlwaysInline(span)),
            TokenType::InlineHint => Some(ThrushAttribute::InlineHint(span)),
            TokenType::Hot => Some(ThrushAttribute::Hot(span)),
            TokenType::SafeStack => Some(ThrushAttribute::SafeStack(span)),
            TokenType::WeakStack => Some(ThrushAttribute::WeakStack(span)),
            TokenType::StrongStack => Some(ThrushAttribute::StrongStack(span)),
            TokenType::PreciseFloats => Some(ThrushAttribute::PreciseFloats(span)),
            TokenType::Stack => Some(ThrushAttribute::Stack(span)),
            TokenType::Heap => Some(ThrushAttribute::Heap(span)),
            TokenType::AsmThrow => Some(ThrushAttribute::AsmThrow(span)),
            TokenType::AsmSideEffects => Some(ThrushAttribute::AsmSideEffects(span)),
            TokenType::AsmAlignStack => Some(ThrushAttribute::AsmAlignStack(span)),
            TokenType::Packed => Some(ThrushAttribute::Packed(span)),
            TokenType::NoUnwind => Some(ThrushAttribute::NoUnwind(span)),
            TokenType::OptFuzzing => Some(ThrushAttribute::OptFuzzing(span)),

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
