#![allow(clippy::upper_case_acronyms)]

use crate::backends::classical::llvm::compiler::conventions::CallConvention;
use crate::frontends::classical::lexer::span::Span;

#[derive(Debug, Clone, Copy)]
pub enum LLVMAttribute<'ctx> {
    // Function Attributes
    Extern(&'ctx str, Span),
    Convention(CallConvention, Span),
    Public(Span),
    Ignore(Span),
    Hot(Span),
    NoInline(Span),
    InlineHint(Span),
    MinSize(Span),
    AlwaysInline(Span),
    SafeStack(Span),
    StrongStack(Span),
    WeakStack(Span),
    PreciseFloats(Span),
    NoUnwind(Span),
    OptFuzzing(Span),

    // LLVM Structure Modificator
    Packed(Span),

    // Memory Management
    Stack(Span),
    Heap(Span),

    // Assembler Attributes
    AsmThrow(Span),
    AsmSyntax(&'ctx str, Span),
    AsmAlignStack(Span),
    AsmSideEffects(Span),
}

impl LLVMAttribute<'_> {
    #[inline]
    pub fn is_extern_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Extern(..))
    }

    #[inline]
    pub fn is_hot_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Hot(..))
    }

    #[inline]
    pub fn is_ignore_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Ignore(..))
    }

    #[inline]
    pub fn is_public_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Public(..))
    }

    #[inline]
    pub fn is_noinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::NoInline(..))
    }

    #[inline]
    pub fn is_inline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::InlineHint(..))
    }

    #[inline]
    pub fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AlwaysInline(..))
    }

    #[inline]
    pub fn is_minsize_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::MinSize(..))
    }

    #[inline]
    pub fn is_heap_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Heap(..))
    }

    #[inline]
    pub fn is_stack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Stack(..))
    }

    #[inline]
    pub fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmSideEffects(..))
    }

    #[inline]
    pub fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmThrow(..))
    }

    #[inline]
    pub fn is_asmalingstack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmAlignStack(..))
    }

    #[inline]
    pub fn is_packed(&self) -> bool {
        matches!(self, LLVMAttribute::Packed(..))
    }
}

impl LLVMAttribute<'_> {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            LLVMAttribute::Extern(_, span) => *span,
            LLVMAttribute::Convention(_, span) => *span,
            LLVMAttribute::Public(span) => *span,
            LLVMAttribute::Ignore(span) => *span,
            LLVMAttribute::Hot(span) => *span,
            LLVMAttribute::NoInline(span) => *span,
            LLVMAttribute::InlineHint(span) => *span,
            LLVMAttribute::MinSize(span) => *span,
            LLVMAttribute::AlwaysInline(span) => *span,
            LLVMAttribute::SafeStack(span) => *span,
            LLVMAttribute::StrongStack(span) => *span,
            LLVMAttribute::WeakStack(span) => *span,
            LLVMAttribute::PreciseFloats(span) => *span,
            LLVMAttribute::AsmThrow(span) => *span,
            LLVMAttribute::AsmSyntax(_, span) => *span,
            LLVMAttribute::AsmSideEffects(span) => *span,
            LLVMAttribute::AsmAlignStack(span) => *span,
            LLVMAttribute::Stack(span) => *span,
            LLVMAttribute::Heap(span) => *span,
            LLVMAttribute::Packed(span) => *span,
            LLVMAttribute::NoUnwind(span) => *span,
            LLVMAttribute::OptFuzzing(span) => *span,
        }
    }
}
