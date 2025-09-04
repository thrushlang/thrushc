#[derive(Debug, Clone, Copy)]
pub struct StaticMetadata {
    is_global: bool,
    is_mutable: bool,

    llvm_metadata: LLVMStaticMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMStaticMetadata {
    pub unnamed_addr: bool,
    pub constant: bool,
    pub thread_local: bool,
    pub volatile: bool,
}

impl StaticMetadata {
    pub fn new(is_global: bool, is_mutable: bool, thread_local: bool, volatile: bool) -> Self {
        Self {
            is_global,
            is_mutable,

            llvm_metadata: LLVMStaticMetadata {
                unnamed_addr: !is_mutable,
                constant: !is_mutable,
                thread_local,
                volatile,
            },
        }
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    #[inline]
    pub fn is_global(&self) -> bool {
        self.is_global
    }

    #[inline]
    pub fn get_llvm_metadata(&self) -> LLVMStaticMetadata {
        self.llvm_metadata
    }
}
