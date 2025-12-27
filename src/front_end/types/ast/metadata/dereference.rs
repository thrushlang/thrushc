#[derive(Debug, Clone, Copy, Default)]
pub struct DereferenceMetadata {
    llvm_metadata: LLVMDereferenceMetadata,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LLVMDereferenceMetadata {
    pub volatile: bool,
    pub atomic_ord: Option<crate::middle_end::mir::atomicord::ThrushAtomicOrdering>,
}

impl DereferenceMetadata {
    #[inline]
    pub fn new(
        is_volatile: bool,
        atomic_ord: Option<crate::middle_end::mir::atomicord::ThrushAtomicOrdering>,
    ) -> Self {
        Self {
            llvm_metadata: LLVMDereferenceMetadata {
                volatile: is_volatile,
                atomic_ord,
            },
        }
    }
}

impl DereferenceMetadata {
    #[inline]
    pub fn get_llvm_metadata(&self) -> LLVMDereferenceMetadata {
        self.llvm_metadata
    }
}
