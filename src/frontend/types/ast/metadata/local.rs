use inkwell::AtomicOrdering;

#[derive(Debug, Clone, Copy)]
pub struct LocalMetadata {
    is_undefined: bool,
    is_mutable: bool,

    llvm_metadata: LLVMLocalMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMLocalMetadata {
    pub volatile: bool,
    pub atomic_ord: Option<AtomicOrdering>,
}

impl LocalMetadata {
    #[inline]
    pub fn new(
        is_undefined: bool,
        is_mutable: bool,
        volatile: bool,
        atomic_ord: Option<AtomicOrdering>,
    ) -> Self {
        Self {
            is_undefined,
            is_mutable,

            llvm_metadata: LLVMLocalMetadata {
                volatile,
                atomic_ord,
            },
        }
    }
}

impl LocalMetadata {
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.is_undefined
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    #[inline]
    pub fn get_llvm_metadata(&self) -> LLVMLocalMetadata {
        self.llvm_metadata
    }
}
