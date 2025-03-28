#![allow(clippy::upper_case_acronyms)]

// Call conventions: https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/IR/CallingConv.h

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CallConvention {
    // " Standard " call conventions.
    Standard = 0,
    Fast = 8,
    Cold = 9,
    Tail = 18,

    // Glasgow Haskell Compiler (GHC)
    GHC = 10,

    // High-Performance Erlang Compiler (HiPE).
    HiPE = 11,

    // Runtime calls that preserves most registers
    PreserveMost = 14,

    // Runtime calls that preserves (almost) all registers
    PreserveAll = 15,

    // Swift.
    Swift = 16,
}
