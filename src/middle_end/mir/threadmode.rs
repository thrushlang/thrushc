#![allow(clippy::enum_variant_names)]

use crate::front_end::lexer::tokentype::TokenType;

#[derive(Debug)]
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

#[inline]
pub fn as_threadmode(token_type: TokenType) -> Option<ThrushThreadMode> {
    match token_type {
        TokenType::ThreadInit => Some(ThrushThreadMode::InitialExecTLSModel),
        TokenType::ThreadDynamic => Some(ThrushThreadMode::GeneralDynamicTLSModel),
        TokenType::ThreadExec => Some(ThrushThreadMode::LocalExecTLSModel),

        _ => None,
    }
}
