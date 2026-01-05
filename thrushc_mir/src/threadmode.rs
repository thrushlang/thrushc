#![allow(clippy::enum_variant_names)]

#[derive(Debug, Clone, Copy)]
pub enum ThrushThreadMode {
    GeneralDynamicTLSModel,
    LocalDynamicTLSModel,
    InitialExecTLSModel,
    LocalExecTLSModel,
}

impl ThrushThreadMode {
    #[inline]
    pub fn as_llvm_threadmode(&self) -> inkwell::ThreadLocalMode {
        match self {
            ThrushThreadMode::GeneralDynamicTLSModel => {
                inkwell::ThreadLocalMode::GeneralDynamicTLSModel
            }
            ThrushThreadMode::LocalDynamicTLSModel => {
                inkwell::ThreadLocalMode::LocalDynamicTLSModel
            }
            ThrushThreadMode::InitialExecTLSModel => inkwell::ThreadLocalMode::InitialExecTLSModel,
            ThrushThreadMode::LocalExecTLSModel => inkwell::ThreadLocalMode::LocalExecTLSModel,
        }
    }
}
