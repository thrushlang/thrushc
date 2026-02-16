use inkwell::module::Linkage;

use thrustc_attributes::{ThrustAttribute, ThrustAttributes};
use thrustc_llvm_callconventions::LLVMCallConvention;

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
pub fn into_llvm_attribute(attribute: &ThrustAttribute) -> Option<LLVMAttribute<'_>> {
    match attribute {
        ThrustAttribute::Extern(external_name, ..) => Some(LLVMAttribute::Extern(external_name)),
        ThrustAttribute::Linkage(linkage, ..) => {
            Some(LLVMAttribute::Linkage(linkage.get_llvm_linkage()))
        }
        ThrustAttribute::Convention(name, ..) => Some(LLVMAttribute::Convention(
            thrustc_llvm_callconventions::get_call_convention(name.as_bytes()),
        )),
        ThrustAttribute::Public(..) => Some(LLVMAttribute::Public),
        ThrustAttribute::Ignore(..) => Some(LLVMAttribute::Ignore),
        ThrustAttribute::Hot(..) => Some(LLVMAttribute::Hot),
        ThrustAttribute::NoInline(..) => Some(LLVMAttribute::NoInline),
        ThrustAttribute::InlineHint(..) => Some(LLVMAttribute::InlineHint),
        ThrustAttribute::MinSize(..) => Some(LLVMAttribute::MinSize),
        ThrustAttribute::AlwaysInline(..) => Some(LLVMAttribute::AlwaysInline),
        ThrustAttribute::SafeStack(..) => Some(LLVMAttribute::SafeStack),
        ThrustAttribute::StrongStack(..) => Some(LLVMAttribute::StrongStack),
        ThrustAttribute::WeakStack(..) => Some(LLVMAttribute::WeakStack),
        ThrustAttribute::PreciseFloats(..) => Some(LLVMAttribute::PreciseFloats),
        ThrustAttribute::AsmThrow(..) => Some(LLVMAttribute::AsmThrow),
        ThrustAttribute::AsmSyntax(syntax, ..) => Some(LLVMAttribute::AsmSyntax(syntax)),
        ThrustAttribute::AsmSideEffects(..) => Some(LLVMAttribute::AsmSideEffects),
        ThrustAttribute::AsmAlignStack(..) => Some(LLVMAttribute::AsmAlignStack),
        ThrustAttribute::Stack(..) => Some(LLVMAttribute::Stack),
        ThrustAttribute::Heap(..) => Some(LLVMAttribute::Heap),
        ThrustAttribute::Packed(..) => Some(LLVMAttribute::Packed),
        ThrustAttribute::NoUnwind(..) => Some(LLVMAttribute::NoUnwind),
        ThrustAttribute::OptFuzzing(..) => Some(LLVMAttribute::OptFuzzing),
        ThrustAttribute::Pure(..) => Some(LLVMAttribute::Pure),
        ThrustAttribute::Thunk(..) => Some(LLVMAttribute::Thunk),
        ThrustAttribute::Constructor(..) => Some(LLVMAttribute::Constructor),
        ThrustAttribute::Destructor(..) => Some(LLVMAttribute::Destructor),
    }
}

pub fn into_llvm_attributes(thrust_attributes: &ThrustAttributes) -> Vec<LLVMAttribute<'_>> {
    let mut llvm_attributes: Vec<LLVMAttribute<'_>> = Vec::with_capacity(thrust_attributes.len());

    for attribute in thrust_attributes.iter() {
        if let Some(llvm_attribute) = self::into_llvm_attribute(attribute) {
            llvm_attributes.push(llvm_attribute);
        }
    }

    llvm_attributes
}
