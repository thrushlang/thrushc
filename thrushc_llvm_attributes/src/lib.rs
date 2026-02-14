use inkwell::module::Linkage;

use thrushc_attributes::{ThrushAttribute, ThrushAttributes};
use thrushc_llvm_callconventions::LLVMCallConvention;

pub mod impls;
pub mod traits;

pub type LLVMAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;

#[derive(Debug, Clone, Copy)]
pub enum LLVMAttribute<'ctx> {
    // Function Attributes
    Extern(&'ctx str),
    Convention(LLVMCallConvention),
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
    Pure,
    Thunk,

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

    Constructor,
    Destructor,
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

    #[inline]
    pub fn is_constructor_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Constructor)
    }

    #[inline]
    pub fn is_destructor_attribute(&self) -> bool {
        matches!(self, LLVMAttribute::Destructor)
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
    Pure,
    Thunk,

    Packed,

    Stack,
    Heap,

    AsmThrow,
    AsmSyntax,
    AsmAlignStack,
    AsmSideEffects,

    Constructor,
    Destructor,
}

#[inline]
pub fn into_llvm_attribute(attribute: &ThrushAttribute) -> Option<LLVMAttribute<'_>> {
    match attribute {
        ThrushAttribute::Extern(external_name, ..) => Some(LLVMAttribute::Extern(external_name)),
        ThrushAttribute::Linkage(linkage, ..) => {
            Some(LLVMAttribute::Linkage(linkage.get_llvm_linkage()))
        }
        ThrushAttribute::Convention(name, ..) => Some(LLVMAttribute::Convention(
            thrushc_llvm_callconventions::get_call_convention(name.as_bytes()),
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
        ThrushAttribute::Pure(..) => Some(LLVMAttribute::Pure),
        ThrushAttribute::Thunk(..) => Some(LLVMAttribute::Thunk),
        ThrushAttribute::Constructor(..) => Some(LLVMAttribute::Constructor),
        ThrushAttribute::Destructor(..) => Some(LLVMAttribute::Destructor),
    }
}

pub fn into_llvm_attributes(thrush_attributes: &ThrushAttributes) -> Vec<LLVMAttribute<'_>> {
    let mut llvm_attributes: Vec<LLVMAttribute<'_>> = Vec::with_capacity(thrush_attributes.len());

    for attribute in thrush_attributes.iter() {
        if let Some(llvm_attribute) = self::into_llvm_attribute(attribute) {
            llvm_attributes.push(llvm_attribute);
        }
    }

    llvm_attributes
}
