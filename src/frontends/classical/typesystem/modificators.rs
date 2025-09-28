#![allow(dead_code)]

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StructureTypeModificator {
    llvm: LLVMStructureTypeModificator,
    gcc: GCCStructureTypeModificator,
}

impl StructureTypeModificator {
    #[inline]
    pub fn new(llvm: LLVMStructureTypeModificator, gcc: GCCStructureTypeModificator) -> Self {
        Self { llvm, gcc }
    }

    #[inline]
    pub fn llvm(&self) -> &LLVMStructureTypeModificator {
        &self.llvm
    }

    #[inline]
    pub fn gcc(&self) -> &GCCStructureTypeModificator {
        &self.gcc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GCCStructureTypeModificator {}

impl GCCStructureTypeModificator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LLVMStructureTypeModificator {
    packed: bool,
}

impl LLVMStructureTypeModificator {
    #[inline]
    pub fn new(packed: bool) -> Self {
        Self { packed }
    }

    #[inline]
    pub fn is_packed(&self) -> bool {
        self.packed
    }
}

impl Display for StructureTypeModificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LLVM: packed = {}, GCC: <none>", self.llvm.packed)
    }
}
