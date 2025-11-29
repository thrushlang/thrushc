use crate::{core::diagnostic::span::Span, front_end::lexer::tokentype::TokenType};

impl TokenType {
    #[must_use]
    pub fn as_attribute(
        self,
        span: Span,
    ) -> Option<crate::middle_end::mir::attributes::ThrushAttribute> {
        match self {
            TokenType::Ignore => Some(crate::middle_end::mir::attributes::ThrushAttribute::Ignore(
                span,
            )),
            TokenType::MinSize => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::MinSize(span))
            }
            TokenType::NoInline => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::NoInline(span))
            }
            TokenType::AlwaysInline => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::AlwaysInline(span))
            }
            TokenType::InlineHint => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::InlineHint(span))
            }
            TokenType::Hot => Some(crate::middle_end::mir::attributes::ThrushAttribute::Hot(
                span,
            )),
            TokenType::SafeStack => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::SafeStack(span))
            }
            TokenType::WeakStack => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::WeakStack(span))
            }
            TokenType::StrongStack => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::StrongStack(span))
            }
            TokenType::PreciseFloats => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::PreciseFloats(span))
            }
            TokenType::Stack => Some(crate::middle_end::mir::attributes::ThrushAttribute::Stack(
                span,
            )),
            TokenType::Heap => Some(crate::middle_end::mir::attributes::ThrushAttribute::Heap(
                span,
            )),
            TokenType::AsmThrow => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmThrow(span))
            }
            TokenType::AsmSideEffects => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmSideEffects(span))
            }
            TokenType::AsmAlignStack => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmAlignStack(span))
            }
            TokenType::Packed => Some(crate::middle_end::mir::attributes::ThrushAttribute::Packed(
                span,
            )),
            TokenType::NoUnwind => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::NoUnwind(span))
            }
            TokenType::OptFuzzing => {
                Some(crate::middle_end::mir::attributes::ThrushAttribute::OptFuzzing(span))
            }

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
