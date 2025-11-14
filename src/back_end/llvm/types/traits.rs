use inkwell::InlineAsmDialect;

use crate::back_end::llvm::compiler::attributes::{LLVMAttribute, LLVMAttributeComparator};

pub trait AssemblerFunctionExtensions {
    fn to_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect;
}

pub trait LLVMAttributesExtensions {
    fn has_extern_attribute(&self) -> bool;
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

    fn get_attr(&self, cmp: LLVMAttributeComparator) -> Option<LLVMAttribute<'_>>;
}

pub trait LLVMAttributeComparatorExtensions {
    fn as_attr_cmp(&self) -> LLVMAttributeComparator;
}
