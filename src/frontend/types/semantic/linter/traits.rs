use super::types::LLVMAttributeComparator;

pub trait LLVMAttributeComparatorExtensions {
    fn into_llvm_attr_cmp(self) -> LLVMAttributeComparator;
}
