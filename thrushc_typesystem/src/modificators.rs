#![allow(dead_code)]

use std::fmt::Display;

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FunctionReferenceTypeModificator {
    llvm: LLVMFunctionReferenceTypeModificator,
    gcc: GCCFunctionReferenceTypeModificator,
}

impl FunctionReferenceTypeModificator {
    #[inline]
    pub fn new(
        llvm: LLVMFunctionReferenceTypeModificator,
        gcc: GCCFunctionReferenceTypeModificator,
    ) -> Self {
        Self { llvm, gcc }
    }
}

impl FunctionReferenceTypeModificator {
    #[inline]
    pub fn llvm(&self) -> &LLVMFunctionReferenceTypeModificator {
        &self.llvm
    }

    #[inline]
    pub fn gcc(&self) -> &GCCFunctionReferenceTypeModificator {
        &self.gcc
    }
}

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LLVMFunctionReferenceTypeModificator {
    ignore_args: bool,
}

impl LLVMFunctionReferenceTypeModificator {
    #[inline]
    pub fn new(ignore_args: bool) -> Self {
        Self { ignore_args }
    }

    #[inline]
    pub fn has_ignore(&self) -> bool {
        self.ignore_args
    }
}

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GCCFunctionReferenceTypeModificator {}

impl GCCFunctionReferenceTypeModificator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
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

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GCCStructureTypeModificator {}

impl GCCStructureTypeModificator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
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
        if self.llvm.packed {
            write!(f, "@packed;")?;
        }

        Ok(())
    }
}

impl Display for FunctionReferenceTypeModificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.llvm.ignore_args {
            write!(f, "@ignore;")?;
        }

        Ok(())
    }
}
