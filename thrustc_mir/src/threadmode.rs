#![allow(clippy::enum_variant_names)]

#[derive(Debug, Clone, Copy)]
pub enum ThrustThreadMode {
    GeneralDynamicTLSModel,
    LocalDynamicTLSModel,
    InitialExecTLSModel,
    LocalExecTLSModel,
}

impl ThrustThreadMode {
    #[inline]
    pub fn as_llvm_threadmode(&self) -> inkwell::ThreadLocalMode {
        match self {
            ThrustThreadMode::GeneralDynamicTLSModel => {
                inkwell::ThreadLocalMode::GeneralDynamicTLSModel
            }
            ThrustThreadMode::LocalDynamicTLSModel => {
                inkwell::ThreadLocalMode::LocalDynamicTLSModel
            }
            ThrustThreadMode::InitialExecTLSModel => inkwell::ThreadLocalMode::InitialExecTLSModel,
            ThrustThreadMode::LocalExecTLSModel => inkwell::ThreadLocalMode::LocalExecTLSModel,
        }
    }
}
