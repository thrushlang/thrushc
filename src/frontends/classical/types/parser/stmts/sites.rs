use crate::backends::classical::llvm::compiler::memory::LLVMAllocationSite;

#[derive(Debug, Clone, Copy)]
pub enum AllocationSite {
    Stack,
    Heap,
    Static,
}

impl AllocationSite {
    pub fn to_llvm_allocation_site(self) -> LLVMAllocationSite {
        match self {
            AllocationSite::Heap => LLVMAllocationSite::Heap,
            AllocationSite::Stack => LLVMAllocationSite::Stack,
            AllocationSite::Static => LLVMAllocationSite::Static,
        }
    }
}
