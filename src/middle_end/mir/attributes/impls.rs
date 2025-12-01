use crate::back_end::llvm_codegen::types::repr::LLVMAttributes;

use crate::core::diagnostic::span::Span;
use crate::middle_end::mir::attributes::traits::{
    ThrushAttributeComparatorExtensions, ThrushAttributesExtensions,
};
use crate::middle_end::mir::attributes::{
    ThrushAttribute, ThrushAttributeComparator, ThrushAttributes,
};

impl ThrushAttributesExtensions for ThrushAttributes {
    #[inline]
    fn has_linkage_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_linkage_attribute())
    }

    #[inline]
    fn has_extern_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_extern_attribute())
    }

    #[inline]
    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    #[inline]
    fn has_heap_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_heap_attribute())
    }

    #[inline]
    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }

    #[inline]
    fn has_hot_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_hot_attribute())
    }

    #[inline]
    fn has_inline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_inline_attribute())
    }

    #[inline]
    fn has_minsize_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_minsize_attribute())
    }

    #[inline]
    fn has_inlinealways_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_alwaysinline_attribute())
    }

    #[inline]
    fn has_noinline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_noinline_attribute())
    }

    #[inline]
    fn has_asmalignstack_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmalingstack_attribute())
    }

    #[inline]
    fn has_asmsideffects_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsideeffects_attribute())
    }

    #[inline]
    fn has_asmthrow_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmthrow_attribute())
    }

    #[inline]
    fn has_asmsyntax_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsyntax_attribute())
    }

    #[inline]
    fn match_attr(&self, cmp: ThrushAttributeComparator) -> Option<Span> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(attr_found.get_span());
        }

        None
    }

    #[inline]
    fn get_attr(&self, cmp: ThrushAttributeComparator) -> Option<ThrushAttribute> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(attr_found.clone());
        }

        None
    }

    #[inline]
    fn as_llvm_attributes(&self) -> LLVMAttributes<'_> {
        let mut llvm_attributes: LLVMAttributes = Vec::with_capacity(self.len());

        for attribute in self {
            if let Some(llvm_attribute) = attribute.as_llvm_attribute() {
                llvm_attributes.push(llvm_attribute);
            }
        }

        llvm_attributes
    }
}

impl ThrushAttributeComparatorExtensions for ThrushAttribute {
    #[inline]
    fn as_attr_cmp(&self) -> ThrushAttributeComparator {
        match self {
            ThrushAttribute::Extern(..) => ThrushAttributeComparator::Extern,
            ThrushAttribute::Convention(..) => ThrushAttributeComparator::Convention,
            ThrushAttribute::Linkage(..) => ThrushAttributeComparator::Linkage,
            ThrushAttribute::Stack(..) => ThrushAttributeComparator::Stack,
            ThrushAttribute::Heap(..) => ThrushAttributeComparator::Heap,
            ThrushAttribute::Public(..) => ThrushAttributeComparator::Public,
            ThrushAttribute::Ignore(..) => ThrushAttributeComparator::Ignore,
            ThrushAttribute::Hot(..) => ThrushAttributeComparator::Hot,
            ThrushAttribute::NoInline(..) => ThrushAttributeComparator::NoInline,
            ThrushAttribute::InlineHint(..) => ThrushAttributeComparator::InlineHint,
            ThrushAttribute::MinSize(..) => ThrushAttributeComparator::MinSize,
            ThrushAttribute::AlwaysInline(..) => ThrushAttributeComparator::AlwaysInline,
            ThrushAttribute::SafeStack(_) => ThrushAttributeComparator::SafeStack,
            ThrushAttribute::StrongStack(..) => ThrushAttributeComparator::StrongStack,
            ThrushAttribute::WeakStack(..) => ThrushAttributeComparator::WeakStack,
            ThrushAttribute::PreciseFloats(..) => ThrushAttributeComparator::PreciseFloats,
            ThrushAttribute::AsmAlignStack(..) => ThrushAttributeComparator::AsmAlignStack,
            ThrushAttribute::AsmSyntax(..) => ThrushAttributeComparator::AsmSyntax,
            ThrushAttribute::AsmThrow(..) => ThrushAttributeComparator::AsmThrow,
            ThrushAttribute::AsmSideEffects(..) => ThrushAttributeComparator::AsmSideEffects,
            ThrushAttribute::Packed(..) => ThrushAttributeComparator::Packed,
            ThrushAttribute::NoUnwind(..) => ThrushAttributeComparator::NoUnwind,
            ThrushAttribute::OptFuzzing(..) => ThrushAttributeComparator::OptFuzzing,
        }
    }
}
