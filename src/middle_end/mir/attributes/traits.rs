use crate::back_end::llvm::types::repr::LLVMAttributes;

use crate::core::diagnostic::span::Span;
use crate::middle_end::mir::attributes::{ThrushAttribute, ThrushAttributeComparator};

pub trait ThrushAttributesExtensions {
    fn has_extern_attribute(&self) -> bool;
    fn has_linkage_attribute(&self) -> bool;
    fn has_ignore_attribute(&self) -> bool;
    fn has_public_attribute(&self) -> bool;
    fn has_hot_attr(&self) -> bool;
    fn has_inline_attr(&self) -> bool;
    fn has_noinline_attr(&self) -> bool;
    fn has_minsize_attr(&self) -> bool;
    fn has_inlinealways_attr(&self) -> bool;

    fn has_heap_attr(&self) -> bool;

    fn has_asmalignstack_attribute(&self) -> bool;
    fn has_asmthrow_attribute(&self) -> bool;
    fn has_asmsideffects_attribute(&self) -> bool;
    fn has_asmsyntax_attribute(&self) -> bool;

    fn match_attr(&self, cmp: ThrushAttributeComparator) -> Option<Span>;
    fn get_attr(&self, cmp: ThrushAttributeComparator) -> Option<ThrushAttribute>;

    fn as_llvm_attributes(&self) -> LLVMAttributes<'_>;
}

pub trait ThrushAttributeComparatorExtensions {
    fn as_attr_cmp(&self) -> ThrushAttributeComparator;
}
