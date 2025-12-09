#![allow(clippy::upper_case_acronyms)]

use inkwell::module::Linkage;

use crate::back_end::llvm_codegen::callconventions::CallConvention;

#[derive(Debug, Clone, Copy)]
pub enum LLVMAttribute<'ctx> {
    // Function Attributes
    Extern(&'ctx str),
    Convention(CallConvention),
    Linkage(Linkage),
    Public,
    Ignore,
    Hot,
    NoInline,
    InlineHint,
    MinSize,
    AlwaysInline,
    SafeStack,
    StrongStack,
    WeakStack,
    PreciseFloats,
    NoUnwind,
    OptFuzzing,

    // LLVM Structure Modificator
    Packed,

    // Memory Management
    Stack,
    Heap,

    // Assembler Attributes
    AsmThrow,
    AsmSyntax(&'ctx str),
    AsmAlignStack,
    AsmSideEffects,
}

impl LLVMAttribute<'_> {
    #[inline]
    pub fn is_extern_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Extern(..))
    }

    #[inline]
    pub fn is_hot_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Hot)
    }

    #[inline]
    pub fn is_ignore_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Ignore)
    }

    #[inline]
    pub fn is_public_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Public)
    }

    #[inline]
    pub fn is_noinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::NoInline)
    }

    #[inline]
    pub fn is_inline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::InlineHint)
    }

    #[inline]
    pub fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AlwaysInline)
    }

    #[inline]
    pub fn is_minsize_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::MinSize)
    }

    #[inline]
    pub fn is_heap_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Heap)
    }

    #[inline]
    pub fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmSideEffects)
    }

    #[inline]
    pub fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmThrow)
    }

    #[inline]
    pub fn is_asmalingstack_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmAlignStack)
    }

    #[inline]
    pub fn is_asmsyntax_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::AsmSyntax(..))
    }

    #[inline]
    pub fn is_packed_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Packed)
    }

    #[inline]
    pub fn is_linkage_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Linkage(..))
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LLVMAttributeComparator {
    Extern,
    Convention,
    Public,
    Ignore,
    Hot,
    NoInline,
    InlineHint,
    MinSize,
    AlwaysInline,
    SafeStack,
    StrongStack,
    WeakStack,
    PreciseFloats,
    NoUnwind,
    OptFuzzing,
    Linkage,

    Stack,
    Heap,

    AsmThrow,
    AsmSyntax,
    AsmAlignStack,
    AsmSideEffects,

    Packed,
}
