use std::fmt::Display;

use crate::{
    back_end::llvm::{compiler::attributes::LLVMAttribute, types::repr::LLVMAttributes},
    front_end::lexer::span::Span,
};

pub mod callconventions;
pub mod linkage;

#[derive(Debug, Clone)]
pub enum ThrushAttribute {
    Extern(String, Span),
    Convention(String, Span),
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

    AsmThrow(Span),
    AsmSyntax(String, Span),
    AsmAlignStack(Span),
    AsmSideEffects(Span),
}

impl ThrushAttribute {
    #[inline]
    pub fn is_extern_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Extern(..))
    }

    #[inline]
    pub fn is_hot_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Hot(..))
    }

    #[inline]
    pub fn is_ignore_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Ignore(..))
    }

    #[inline]
    pub fn is_public_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Public(..))
    }

    #[inline]
    pub fn is_noinline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::NoInline(..))
    }

    #[inline]
    pub fn is_inline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::InlineHint(..))
    }

    #[inline]
    pub fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AlwaysInline(..))
    }

    #[inline]
    pub fn is_minsize_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::MinSize(..))
    }

    #[inline]
    pub fn is_heap_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Heap(..))
    }

    #[inline]
    pub fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmSideEffects(..))
    }

    #[inline]
    pub fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmThrow(..))
    }

    #[inline]
    pub fn is_asmalingstack_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmAlignStack(..))
    }

    #[inline]
    pub fn is_asmsyntax_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmSyntax(..))
    }

    #[inline]
    pub fn is_packed(&self) -> bool {
        matches!(self, ThrushAttribute::Packed(..))
    }
}

impl ThrushAttribute {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            ThrushAttribute::Extern(_, span) => *span,
            ThrushAttribute::Convention(_, span) => *span,
            ThrushAttribute::Public(span) => *span,
            ThrushAttribute::Ignore(span) => *span,
            ThrushAttribute::Hot(span) => *span,
            ThrushAttribute::NoInline(span) => *span,
            ThrushAttribute::InlineHint(span) => *span,
            ThrushAttribute::MinSize(span) => *span,
            ThrushAttribute::AlwaysInline(span) => *span,
            ThrushAttribute::SafeStack(span) => *span,
            ThrushAttribute::StrongStack(span) => *span,
            ThrushAttribute::WeakStack(span) => *span,
            ThrushAttribute::PreciseFloats(span) => *span,
            ThrushAttribute::AsmThrow(span) => *span,
            ThrushAttribute::AsmSyntax(_, span) => *span,
            ThrushAttribute::AsmSideEffects(span) => *span,
            ThrushAttribute::AsmAlignStack(span) => *span,
            ThrushAttribute::Stack(span) => *span,
            ThrushAttribute::Heap(span) => *span,
            ThrushAttribute::Packed(span) => *span,
            ThrushAttribute::NoUnwind(span) => *span,
            ThrushAttribute::OptFuzzing(span) => *span,
        }
    }
}

impl ThrushAttribute {
    #[inline]
    pub fn as_llvm_attribute(&self) -> Option<LLVMAttribute<'_>> {
        match self {
            ThrushAttribute::Extern(external_name, ..) => {
                Some(LLVMAttribute::Extern(external_name))
            }
            ThrushAttribute::Convention(name, ..) => Some(LLVMAttribute::Convention(
                callconventions::get_call_convention(name.as_bytes()),
            )),
            ThrushAttribute::Public(..) => Some(LLVMAttribute::Public),
            ThrushAttribute::Ignore(..) => Some(LLVMAttribute::Ignore),
            ThrushAttribute::Hot(..) => Some(LLVMAttribute::Hot),
            ThrushAttribute::NoInline(..) => Some(LLVMAttribute::NoInline),
            ThrushAttribute::InlineHint(..) => Some(LLVMAttribute::InlineHint),
            ThrushAttribute::MinSize(..) => Some(LLVMAttribute::MinSize),
            ThrushAttribute::AlwaysInline(..) => Some(LLVMAttribute::AlwaysInline),
            ThrushAttribute::SafeStack(..) => Some(LLVMAttribute::SafeStack),
            ThrushAttribute::StrongStack(..) => Some(LLVMAttribute::StrongStack),
            ThrushAttribute::WeakStack(..) => Some(LLVMAttribute::WeakStack),
            ThrushAttribute::PreciseFloats(..) => Some(LLVMAttribute::PreciseFloats),
            ThrushAttribute::AsmThrow(..) => Some(LLVMAttribute::AsmThrow),
            ThrushAttribute::AsmSyntax(syntax, ..) => Some(LLVMAttribute::AsmSyntax(syntax)),
            ThrushAttribute::AsmSideEffects(..) => Some(LLVMAttribute::AsmSideEffects),
            ThrushAttribute::AsmAlignStack(..) => Some(LLVMAttribute::AsmAlignStack),
            ThrushAttribute::Stack(..) => Some(LLVMAttribute::Stack),
            ThrushAttribute::Heap(..) => Some(LLVMAttribute::Heap),
            ThrushAttribute::Packed(..) => Some(LLVMAttribute::Packed),
            ThrushAttribute::NoUnwind(..) => Some(LLVMAttribute::NoUnwind),
            ThrushAttribute::OptFuzzing(..) => Some(LLVMAttribute::OptFuzzing),
        }
    }
}

impl Display for ThrushAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrushAttribute::AlwaysInline(..) => write!(f, "@alwaysinline"),
            ThrushAttribute::NoInline(..) => write!(f, "@noinline"),
            ThrushAttribute::InlineHint(..) => write!(f, "@inline"),
            ThrushAttribute::Extern(name, ..) => write!(f, "@extern({})", name),
            ThrushAttribute::Convention(convention, ..) => {
                write!(f, "@convention(\"{}\")", convention)
            }
            ThrushAttribute::Stack(..) => write!(f, "@stack"),
            ThrushAttribute::Heap(..) => write!(f, "@heap"),
            ThrushAttribute::Public(..) => write!(f, "@public"),
            ThrushAttribute::StrongStack(..) => write!(f, "@strongstack"),
            ThrushAttribute::WeakStack(..) => write!(f, "@weakstack"),
            ThrushAttribute::SafeStack(..) => write!(f, "@safestack"),
            ThrushAttribute::PreciseFloats(..) => write!(f, "@precisefp"),
            ThrushAttribute::MinSize(..) => write!(f, "@minsize"),
            ThrushAttribute::Hot(..) => write!(f, "@hot"),
            ThrushAttribute::Ignore(..) => write!(f, "@ignore"),
            ThrushAttribute::NoUnwind(..) => write!(f, "@nounwind"),
            ThrushAttribute::AsmThrow(..) => write!(f, "@asmthrow"),
            ThrushAttribute::AsmSyntax(..) => write!(f, "@asmsyntax"),
            ThrushAttribute::AsmSideEffects(..) => write!(f, "@asmeffects"),
            ThrushAttribute::AsmAlignStack(..) => write!(f, "@asmalingstack"),
            ThrushAttribute::Packed(..) => write!(f, "@packed"),
            ThrushAttribute::OptFuzzing(..) => write!(f, "@optfuzzing"),
        }
    }
}
