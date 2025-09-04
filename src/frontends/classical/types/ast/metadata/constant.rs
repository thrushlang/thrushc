#[derive(Debug, Clone, Copy)]
pub struct ConstantMetadata {
    is_global: bool,

    llvm_metadata: LLVMConstantMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMConstantMetadata {
    pub thread_local: bool,
    pub volatile: bool,
}

impl ConstantMetadata {
    pub fn new(is_global: bool, thread_local: bool, volatile: bool) -> Self {
        Self {
            is_global,

            llvm_metadata: LLVMConstantMetadata {
                thread_local,
                volatile,
            },
        }
    }

    #[inline]
    pub fn is_global(&self) -> bool {
        self.is_global
    }

    #[inline]
    pub fn get_llvm_metadata(&self) -> LLVMConstantMetadata {
        self.llvm_metadata
    }
}
