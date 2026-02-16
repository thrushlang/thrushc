use thrustc_span::Span;

use crate::{ThrustAttribute, ThrustAttributeComparator};

pub trait ThrustAttributesExtensions {
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
    fn has_constructor_attribute(&self) -> bool;
    fn has_destructor_attribute(&self) -> bool;
    fn has_asmsyntax_attribute(&self) -> bool;
    fn has_convention_attribute(&self) -> bool;

    fn match_attr(&self, cmp: ThrustAttributeComparator) -> Option<Span>;
    fn get_attr(&self, cmp: ThrustAttributeComparator) -> Option<ThrustAttribute>;

    //fn as_llvm_attributes(&self) -> LLVMAttributes<'_>;
}

pub trait ThrustAttributeComparatorExtensions {
    fn as_attr_cmp(&self) -> ThrustAttributeComparator;
}
