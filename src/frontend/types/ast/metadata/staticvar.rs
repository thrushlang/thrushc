#[derive(Debug, Clone, Copy)]
pub struct StaticMetadata {
    is_global: bool,
    is_mutable: bool,

    llvm_metadata: LLVMStaticMetadata,
}

#[derive(Debug, Clone, Copy)]
pub struct LLVMStaticMetadata {
    pub can_unnamed_addr: bool,
    pub can_constant: bool,
}

impl StaticMetadata {
    pub fn new(is_global: bool, is_mutable: bool) -> Self {
        Self {
            is_global,
            is_mutable,

            llvm_metadata: LLVMStaticMetadata {
                can_unnamed_addr: !is_mutable,
                can_constant: !is_mutable,
            },
        }
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn get_llvm_metadata(&self) -> LLVMStaticMetadata {
        self.llvm_metadata
    }
}
