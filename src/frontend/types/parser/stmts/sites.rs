use crate::backend::llvm::compiler::memory::LLVMAllocationSite;

#[derive(Debug, Clone, Copy)]
pub enum LLIAllocationSite {
    Stack,
    Heap,
    Static,
}

impl LLIAllocationSite {
    pub fn to_llvm_allocation_site(self) -> LLVMAllocationSite {
        match self {
            LLIAllocationSite::Heap => LLVMAllocationSite::Heap,
            LLIAllocationSite::Stack => LLVMAllocationSite::Stack,
            LLIAllocationSite::Static => LLVMAllocationSite::Static,
        }
    }
}
