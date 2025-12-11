#![allow(clippy::upper_case_acronyms, non_camel_case_types)]

// Call conventions: https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/IR/CallingConv.h

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
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

    // Dynamic register based calls
    AnyReg = 13,

    // Preserves most caller-saved registers, balancing performance and compatibility.
    PreserveMost = 14,

    // Preserves all caller-saved registers, ensuring maximum compatibility but with higher overhead.
    PreserveAll = 15,

    // Swift.
    Swift = 16,

    // Access functions
    CXX_FAST_TLS = 17,

    // Control Guard Check ICall function
    CFGuard_Check = 19,

    // Swift with tail call guarantee
    SwiftTail = 20,

    // Preserves no general registers
    PreserveNone = 21,

    // X86 stdcall
    X86_StdCall = 64,

    // X86 fastcall
    X86_FastCall = 65,

    // ARM APCS
    ARM_APCS = 66,

    // ARM AAPCS
    ARM_AAPCS = 67,

    // ARM AAPCS VFP
    ARM_AAPCS_VFP = 68,

    // MSP430 interrupt
    MSP430_INTR = 69,

    // X86 thiscall
    X86_ThisCall = 70,

    // PTX kernel
    PTX_Kernel = 71,

    // PTX device
    PTX_Device = 72,

    // SPIR function
    SPIR_FUNC = 75,

    // SPIR kernel
    SPIR_KERNEL = 76,

    // Intel OpenCL built-ins
    Intel_OCL_BI = 77,

    // x86-64 System V
    X86_64_SysV = 78,

    // Win64
    Win64 = 79,

    // X86 vector call
    X86_VectorCall = 80,

    // X86 interrupt
    X86_INTR = 83,

    // AVR interrupt
    AVR_INTR = 84,

    // AVR signal
    AVR_SIGNAL = 85,

    // AVR builtin
    AVR_BUILTIN = 86,

    // AMDGPU vertex shader
    AMDGPU_VS = 87,

    // AMDGPU geometry shader
    AMDGPU_GS = 88,

    // AMDGPU pixel shader
    AMDGPU_PS = 89,

    // AMDGPU compute shader
    AMDGPU_CS = 90,

    // AMDGPU kernel
    AMDGPU_KERNEL = 91,

    // X86 register call
    X86_RegCall = 92,

    // AMDGPU hull shader
    AMDGPU_HS = 93,

    // MSP430 builtin
    MSP430_BUILTIN = 94,

    // AMDGPU local shader
    AMDGPU_LS = 95,

    // AMDGPU export shader
    AMDGPU_ES = 96,

    // AArch64 vector call
    AArch64_VectorCall = 97,

    // AArch64 SVE vector call
    AArch64_SVE_VectorCall = 98,

    // WebAssembly Emscripten invoke
    WASM_EmscriptenInvoke = 99,

    // AMDGPU graphics
    AMDGPU_Gfx = 100,

    // M68k interrupt
    M68k_INTR = 101,

    // AArch64 SME ABI support routines preserve most from X0
    AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0 = 102,

    // AArch64 SME ABI support routines preserve most from X2
    AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2 = 103,

    // AMDGPU CS chain
    AMDGPU_CS_Chain = 104,

    // AMDGPU CS chain preserve
    AMDGPU_CS_ChainPreserve = 105,

    // M68k RTD
    M68k_RTD = 106,

    // GraalVM
    GraalVM = 107,

    // ARM64EC thunk x64
    ARM64EC_Thunk_X64 = 108,

    // ARM64EC thunk native
    ARM64EC_Thunk_Native = 109,

    // RISC-V vector call
    RISCV_VectorCall = 110,

    // AArch64 SME ABI support routines preserve most from X1
    AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1 = 111,

    // RISC-V VLS calls
    RISCV_VLSCall_32 = 112,
    RISCV_VLSCall_64 = 113,
    RISCV_VLSCall_128 = 114,
    RISCV_VLSCall_256 = 115,
    RISCV_VLSCall_512 = 116,
    RISCV_VLSCall_1024 = 117,
    RISCV_VLSCall_2048 = 118,
    RISCV_VLSCall_4096 = 119,
    RISCV_VLSCall_8192 = 120,
    RISCV_VLSCall_16384 = 121,
    RISCV_VLSCall_32768 = 122,
    RISCV_VLSCall_65536 = 123,

    // AMDGPU graphics whole wave
    AMDGPU_Gfx_WholeWave = 124,

    // CHERIoT compartment call
    CHERIoT_CompartmentCall = 125,

    // CHERIoT compartment callee
    CHERIoT_CompartmentCallee = 126,

    // CHERIoT library call
    CHERIoT_LibraryCall = 127,
}

impl CallConvention {
    #[inline]
    pub fn is_specific_target_conv(&self) -> bool {
        matches!(
            self,
            CallConvention::AMDGPU_CS
                | CallConvention::AMDGPU_CS_Chain
                | CallConvention::AMDGPU_CS_ChainPreserve
                | CallConvention::AMDGPU_ES
                | CallConvention::AMDGPU_GS
                | CallConvention::AMDGPU_Gfx
                | CallConvention::AMDGPU_Gfx_WholeWave
                | CallConvention::AMDGPU_HS
                | CallConvention::AMDGPU_KERNEL
                | CallConvention::AMDGPU_LS
                | CallConvention::AMDGPU_PS
                | CallConvention::AMDGPU_VS
                | CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0
                | CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1
                | CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2
                | CallConvention::AArch64_SVE_VectorCall
                | CallConvention::AArch64_VectorCall
                | CallConvention::RISCV_VLSCall_1024
                | CallConvention::RISCV_VLSCall_128
                | CallConvention::RISCV_VLSCall_16384
                | CallConvention::RISCV_VLSCall_2048
                | CallConvention::RISCV_VLSCall_256
                | CallConvention::RISCV_VLSCall_32
                | CallConvention::RISCV_VLSCall_32768
                | CallConvention::RISCV_VLSCall_4096
                | CallConvention::RISCV_VLSCall_512
                | CallConvention::RISCV_VLSCall_64
                | CallConvention::RISCV_VLSCall_65536
                | CallConvention::RISCV_VLSCall_8192
                | CallConvention::RISCV_VectorCall
                | CallConvention::ARM_AAPCS
                | CallConvention::ARM_AAPCS_VFP
                | CallConvention::ARM_APCS
                | CallConvention::ARM64EC_Thunk_Native
                | CallConvention::ARM64EC_Thunk_X64
                | CallConvention::X86_StdCall
                | CallConvention::X86_FastCall
                | CallConvention::X86_ThisCall
                | CallConvention::X86_64_SysV
                | CallConvention::X86_INTR
                | CallConvention::X86_VectorCall
                | CallConvention::X86_RegCall
        )
    }
}
