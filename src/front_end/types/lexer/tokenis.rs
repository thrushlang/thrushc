use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::lexer::traits::{TokenTypeBuiltinExtensions, TokenTypeExtensions};

impl TokenTypeExtensions for TokenType {
    #[inline]
    fn is_logical_operator(&self) -> bool {
        matches!(
            self,
            TokenType::BangEq
                | TokenType::EqEq
                | TokenType::LessEq
                | TokenType::Less
                | TokenType::Greater
                | TokenType::GreaterEq
        )
    }

    #[inline]
    fn is_logical_gate(&self) -> bool {
        matches!(self, TokenType::And | TokenType::Or)
    }

    #[inline]
    fn is_minus_minus_operator(&self) -> bool {
        matches!(self, TokenType::MinusMinus)
    }

    #[inline]
    fn is_plus_plus_operator(&self) -> bool {
        matches!(self, TokenType::PlusPlus)
    }

    #[inline]
    fn is_address(&self) -> bool {
        matches!(self, TokenType::Addr)
    }

    #[inline]
    fn is_void(&self) -> bool {
        matches!(self, TokenType::Void)
    }

    #[inline]
    fn is_bool(&self) -> bool {
        matches!(self, TokenType::Bool)
    }

    #[inline]
    fn is_array(&self) -> bool {
        matches!(self, TokenType::Array)
    }

    #[inline]
    fn is_ptr(&self) -> bool {
        matches!(self, TokenType::Ptr)
    }

    #[inline]
    fn is_float(&self) -> bool {
        matches!(
            self,
            TokenType::F32
                | TokenType::F64
                | TokenType::F128
                | TokenType::FX8680
                | TokenType::FPPC128
        )
    }

    #[inline]
    fn is_const(&self) -> bool {
        matches!(self, TokenType::Const)
    }

    #[inline]
    fn is_fn_ref(&self) -> bool {
        matches!(self, TokenType::FnRef)
    }

    #[inline]
    fn is_integer(&self) -> bool {
        matches!(
            self,
            TokenType::S8
                | TokenType::S16
                | TokenType::S32
                | TokenType::S64
                | TokenType::Ssize
                | TokenType::U8
                | TokenType::U16
                | TokenType::U32
                | TokenType::U64
                | TokenType::U128
                | TokenType::Usize
                | TokenType::Char
        )
    }

    #[inline]
    fn is_type(&self) -> bool {
        self.is_integer()
            || self.is_float()
            || self.is_bool()
            || self.is_array()
            || self.is_ptr()
            || self.is_void()
            || self.is_address()
            || self.is_const()
            || self.is_fn_ref()
    }

    #[inline]
    fn is_attribute(&self) -> bool {
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
                | TokenType::AsmSyntax
                | TokenType::Packed
                | TokenType::NoUnwind
                | TokenType::OptFuzzing
                | TokenType::Constructor
                | TokenType::Destructor
                | TokenType::Public
                | TokenType::Linkage
                | TokenType::Extern
                | TokenType::Convention
        )
    }

    #[inline]
    fn is_identifier(&self) -> bool {
        matches!(self, TokenType::Identifier)
    }
}

impl TokenTypeBuiltinExtensions for TokenType {
    fn is_builtin(&self) -> bool {
        matches!(
            self,
            TokenType::Halloc
                | TokenType::MemCpy
                | TokenType::MemMove
                | TokenType::MemSet
                | TokenType::AlignOf
                | TokenType::SizeOf
                | TokenType::BitSizeOf
                | TokenType::AbiSizeOf
                | TokenType::AbiAlignOf
        )
    }
}
