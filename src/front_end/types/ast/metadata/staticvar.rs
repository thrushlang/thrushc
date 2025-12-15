use inkwell::{AtomicOrdering, ThreadLocalMode};

#[derive(Debug, Clone, Copy)]
pub struct StaticMetadata {
    is_global: bool,
    is_mutable: bool,
    is_unitialized: bool,

    llvm_metadata: LLVMStaticMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMStaticMetadata {
    pub unnamed_addr: bool,
    pub constant: bool,
    pub thread_local: bool,
    pub thread_mode: Option<ThreadLocalMode>,
    pub volatile: bool,
    pub atomic_ord: Option<AtomicOrdering>,
}

impl StaticMetadata {
    #[inline]
    pub fn new(
        is_global: bool,
        is_mutable: bool,
        is_unitialized: bool,
        thread_local: bool,
        volatile: bool,
        external: bool,
        atomic_ord: Option<AtomicOrdering>,
        thread_mode: Option<ThreadLocalMode>,
    ) -> Self {
        Self {
            is_global,
            is_mutable,
            is_unitialized,

            llvm_metadata: LLVMStaticMetadata {
                unnamed_addr: !is_mutable && !external,
                constant: !is_mutable,
                thread_local,
                thread_mode,
                volatile,
                atomic_ord,
            },
        }
    }
}

impl StaticMetadata {
    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    #[inline]
    pub fn is_unitialized(&self) -> bool {
        self.is_unitialized
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
