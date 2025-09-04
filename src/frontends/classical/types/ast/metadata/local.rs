#[derive(Debug, Clone, Copy)]
pub struct LocalMetadata {
    is_undefined: bool,
    is_mutable: bool,

    llvm_metadata: LLVMLocalMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMLocalMetadata {
    pub volatile: bool,
}

impl LocalMetadata {
    pub fn new(is_undefined: bool, is_mutable: bool, volatile: bool) -> Self {
        Self {
            is_undefined,
            is_mutable,

            llvm_metadata: LLVMLocalMetadata { volatile },
        }
    }

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
