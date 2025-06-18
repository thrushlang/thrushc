use crate::backend::llvm::compiler::attributes::LLVMAttribute;

use super::{traits::LLVMAttributeComparatorExtensions, types::LLVMAttributeComparator};

impl LLVMAttributeComparatorExtensions for LLVMAttribute<'_> {
    fn into_llvm_attr_cmp(self) -> LLVMAttributeComparator {
        match self {
            LLVMAttribute::Extern(..) => LLVMAttributeComparator::Extern,
            LLVMAttribute::Convention(..) => LLVMAttributeComparator::Convention,
            LLVMAttribute::Stack(..) => LLVMAttributeComparator::Stack,
            LLVMAttribute::Heap(..) => LLVMAttributeComparator::Heap,
            LLVMAttribute::Public(..) => LLVMAttributeComparator::Public,
            LLVMAttribute::Ignore(..) => LLVMAttributeComparator::Ignore,
            LLVMAttribute::Hot(..) => LLVMAttributeComparator::Hot,
            LLVMAttribute::NoInline(..) => LLVMAttributeComparator::NoInline,
            LLVMAttribute::InlineHint(..) => LLVMAttributeComparator::InlineHint,
            LLVMAttribute::MinSize(..) => LLVMAttributeComparator::MinSize,
            LLVMAttribute::AlwaysInline(..) => LLVMAttributeComparator::AlwaysInline,
            LLVMAttribute::SafeStack(_) => LLVMAttributeComparator::SafeStack,
            LLVMAttribute::StrongStack(..) => LLVMAttributeComparator::StrongStack,
            LLVMAttribute::WeakStack(..) => LLVMAttributeComparator::WeakStack,
            LLVMAttribute::PreciseFloats(..) => LLVMAttributeComparator::PreciseFloats,
            LLVMAttribute::AsmAlignStack(..) => LLVMAttributeComparator::AsmAlignStack,
            LLVMAttribute::AsmSyntax(..) => LLVMAttributeComparator::AsmSyntax,
            LLVMAttribute::AsmThrow(..) => LLVMAttributeComparator::AsmThrow,
            LLVMAttribute::AsmSideEffects(..) => LLVMAttributeComparator::AsmSideEffects,
        }
    }
}
