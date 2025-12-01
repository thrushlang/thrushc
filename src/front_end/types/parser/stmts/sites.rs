use crate::back_end::llvm_codegen::memory::LLVMAllocationSite;

#[derive(Debug, Clone, Copy)]
pub enum AllocationSite {
    Stack,
    Heap,
    Static,
}

impl AllocationSite {
    #[inline]
    pub fn to_llvm_allocation_site(self) -> LLVMAllocationSite {
        match self {
            AllocationSite::Heap => LLVMAllocationSite::Heap,
            AllocationSite::Stack => LLVMAllocationSite::Stack,
            AllocationSite::Static => LLVMAllocationSite::Static,
        }
    }
}
