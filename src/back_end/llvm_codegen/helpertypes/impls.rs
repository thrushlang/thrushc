use crate::back_end::llvm_codegen::attributes::LLVMAttribute;
use crate::back_end::llvm_codegen::attributes::LLVMAttributeComparator;
use crate::back_end::llvm_codegen::callconventions::CallConvention;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMAttributes;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMDBGFunction;
use crate::back_end::llvm_codegen::helpertypes::repr::LLVMFunction;
use crate::back_end::llvm_codegen::helpertypes::traits::AssemblerFunctionExtensions;
use crate::back_end::llvm_codegen::helpertypes::traits::LLVMAttributeComparatorExtensions;
use crate::back_end::llvm_codegen::helpertypes::traits::LLVMAttributesExtensions;
use crate::back_end::llvm_codegen::helpertypes::traits::LLVMDBGFunctionExtensions;
use crate::back_end::llvm_codegen::helpertypes::traits::LLVMFunctionExtensions;
use crate::back_end::llvm_codegen::helpertypes::traits::LLVMLinkageExtensions;
use crate::core::diagnostic::span::Span;
use crate::front_end::typesystem::types::Type;

use std::fmt::Display;

use inkwell::InlineAsmDialect;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;

impl<'ctx> LLVMFunctionExtensions<'ctx> for LLVMFunction<'ctx> {
    #[inline]
    fn get_value(&self) -> FunctionValue<'ctx> {
        self.0
    }

    #[inline]
    fn get_return_type(&self) -> &'ctx Type {
        self.1
    }

    #[inline]
    fn get_call_convention(&self) -> u32 {
        self.3
    }

    #[inline]
    fn get_param_count(&self) -> usize {
        self.2.len()
    }

    #[inline]
    fn get_parameters_types(&self) -> &[Type] {
        self.2
    }
}

impl<'ctx> LLVMDBGFunctionExtensions<'ctx> for LLVMDBGFunction<'ctx> {
    #[inline]
    fn get_name(&self) -> &str {
        &self.0
    }

    #[inline]
    fn get_value(&self) -> FunctionValue<'ctx> {
        self.1
    }

    #[inline]
    fn get_return_type(&self) -> &'ctx Type {
        self.2
    }

    #[inline]
    fn get_parameters_types(&self) -> &[Type] {
        self.3
    }

    #[inline]
    fn is_definition(&self) -> bool {
        self.4
    }

    #[inline]
    fn is_local(&self) -> bool {
        self.5
    }

    #[inline]
    fn get_span(&self) -> Span {
        self.6
    }
}

impl LLVMAttributesExtensions for LLVMAttributes<'_> {
    #[inline]
    fn has_extern_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_extern_attribute())
    }

    #[inline]
    fn has_linkage_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_linkage_attribute())
    }

    #[inline]
    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    #[inline]
    fn has_heap_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_heap_attribute())
    }

    #[inline]
    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }

    #[inline]
    fn has_hot_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_hot_attribute())
    }

    #[inline]
    fn has_inline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_inline_attribute())
    }

    #[inline]
    fn has_minsize_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_minsize_attribute())
    }

    #[inline]
    fn has_inlinealways_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_alwaysinline_attribute())
    }

    #[inline]
    fn has_noinline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_noinline_attribute())
    }

    #[inline]
    fn has_asmalignstack_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmalingstack_attribute())
    }

    #[inline]
    fn has_asmsideffects_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsideeffects_attribute())
    }

    #[inline]
    fn has_asmthrow_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmthrow_attribute())
    }

    #[inline]
    fn has_asmsyntax_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsyntax_attribute())
    }

    #[inline]
    fn get_attr(&self, cmp: LLVMAttributeComparator) -> Option<LLVMAttribute<'_>> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(*attr_found);
        }

        None
    }
}

impl LLVMAttributeComparatorExtensions for LLVMAttribute<'_> {
    fn as_attr_cmp(&self) -> LLVMAttributeComparator {
        match self {
            LLVMAttribute::Extern(..) => LLVMAttributeComparator::Extern,
            LLVMAttribute::Linkage(..) => LLVMAttributeComparator::Linkage,
            LLVMAttribute::Convention(..) => LLVMAttributeComparator::Convention,
            LLVMAttribute::Stack => LLVMAttributeComparator::Stack,
            LLVMAttribute::Heap => LLVMAttributeComparator::Heap,
            LLVMAttribute::Public => LLVMAttributeComparator::Public,
            LLVMAttribute::Ignore => LLVMAttributeComparator::Ignore,
            LLVMAttribute::Hot => LLVMAttributeComparator::Hot,
            LLVMAttribute::NoInline => LLVMAttributeComparator::NoInline,
            LLVMAttribute::InlineHint => LLVMAttributeComparator::InlineHint,
            LLVMAttribute::MinSize => LLVMAttributeComparator::MinSize,
            LLVMAttribute::AlwaysInline => LLVMAttributeComparator::AlwaysInline,
            LLVMAttribute::SafeStack => LLVMAttributeComparator::SafeStack,
            LLVMAttribute::StrongStack => LLVMAttributeComparator::StrongStack,
            LLVMAttribute::WeakStack => LLVMAttributeComparator::WeakStack,
            LLVMAttribute::PreciseFloats => LLVMAttributeComparator::PreciseFloats,
            LLVMAttribute::AsmAlignStack => LLVMAttributeComparator::AsmAlignStack,
            LLVMAttribute::AsmSyntax(..) => LLVMAttributeComparator::AsmSyntax,
            LLVMAttribute::AsmThrow => LLVMAttributeComparator::AsmThrow,
            LLVMAttribute::AsmSideEffects => LLVMAttributeComparator::AsmSideEffects,
            LLVMAttribute::Packed => LLVMAttributeComparator::Packed,
            LLVMAttribute::NoUnwind => LLVMAttributeComparator::NoUnwind,
            LLVMAttribute::OptFuzzing => LLVMAttributeComparator::OptFuzzing,
            LLVMAttribute::Constructor => LLVMAttributeComparator::Constructor,
            LLVMAttribute::Destructor => LLVMAttributeComparator::Destructor,
        }
    }
}

impl Display for LLVMAttribute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMAttribute::AlwaysInline => write!(f, "@alwaysinline"),
            LLVMAttribute::NoInline => write!(f, "@noinline"),
            LLVMAttribute::InlineHint => write!(f, "@inline"),
            LLVMAttribute::Extern(name, ..) => write!(f, "@extern({})", name),
            LLVMAttribute::Linkage(linkage, ..) => write!(f, "@linkage(\"{}\")", linkage.fmt()),
            LLVMAttribute::Convention(convention, ..) => {
                write!(f, "@convention(\"{}\")", convention)
            }
            LLVMAttribute::Stack => write!(f, "@stack"),
            LLVMAttribute::Heap => write!(f, "@heap"),
            LLVMAttribute::Public => write!(f, "@public"),
            LLVMAttribute::StrongStack => write!(f, "@strongstack"),
            LLVMAttribute::WeakStack => write!(f, "@weakstack"),
            LLVMAttribute::SafeStack => write!(f, "@safestack"),
            LLVMAttribute::PreciseFloats => write!(f, "@precisefp"),
            LLVMAttribute::MinSize => write!(f, "@minsize"),
            LLVMAttribute::Hot => write!(f, "@hot"),
            LLVMAttribute::Ignore => write!(f, "@ignore"),
            LLVMAttribute::NoUnwind => write!(f, "@nounwind"),
            LLVMAttribute::AsmThrow => write!(f, "@asmthrow"),
            LLVMAttribute::AsmSyntax(..) => write!(f, "@asmsyntax"),
            LLVMAttribute::AsmSideEffects => write!(f, "@asmeffects"),
            LLVMAttribute::AsmAlignStack => write!(f, "@asmalingstack"),
            LLVMAttribute::Packed => write!(f, "@packed"),
            LLVMAttribute::OptFuzzing => write!(f, "@optfuzzing"),
            LLVMAttribute::Constructor => write!(f, "@constructor"),
            LLVMAttribute::Destructor => write!(f, "@destructor"),
        }
    }
}

impl Display for CallConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallConvention::Standard => write!(f, "C"),
            CallConvention::Fast => write!(f, "fast"),
            CallConvention::Cold => write!(f, "cold"),
            CallConvention::GHC => write!(f, "Haskell"),
            CallConvention::PreserveAll => write!(f, "strongReg"),
            CallConvention::PreserveMost => write!(f, "weakReg"),
            CallConvention::Tail => write!(f, "tail"),
            CallConvention::Swift => write!(f, "Swift"),
            CallConvention::HiPE => write!(f, "Erlang"),
            CallConvention::GraalVM => write!(f, "GraalVM"),
            CallConvention::Win64 => write!(f, "Win64"),
            CallConvention::X86_StdCall => write!(f, "X86_StdCall"),
            CallConvention::X86_FastCall => write!(f, "X86_FastCall"),
            CallConvention::X86_ThisCall => write!(f, "X86_ThisCall"),
            CallConvention::X86_VectorCall => write!(f, "X86_VectorCall"),
            CallConvention::X86_RegCall => write!(f, "X86_RegCall"),
            CallConvention::X86_64_SysV => write!(f, "X86_64_SysV"),
            CallConvention::ARM_APCS => write!(f, "ARM_APCS"),
            CallConvention::ARM_AAPCS => write!(f, "ARM_AAPCS"),
            CallConvention::ARM_AAPCS_VFP => write!(f, "ARM_AAPCS_VFP"),
            CallConvention::AArch64_VectorCall => write!(f, "AArch64_VectorCall"),
            CallConvention::AArch64_SVE_VectorCall => write!(f, "AArch64_SVE_VectorCall"),
            CallConvention::SwiftTail => write!(f, "SwiftTail"),
            CallConvention::PreserveNone => write!(f, "PreserveNone"),
            CallConvention::AnyReg => write!(f, "AnyReg"),
            CallConvention::PTX_Kernel => write!(f, "PTX_Kernel"),
            CallConvention::PTX_Device => write!(f, "PTX_Device"),
            CallConvention::AMDGPU_KERNEL => write!(f, "AMDGPU_KERNEL"),
            CallConvention::AMDGPU_Gfx => write!(f, "AMDGPU_Gfx"),
            CallConvention::RISCV_VectorCall => write!(f, "RISCV_VectorCall"),
            CallConvention::CXX_FAST_TLS => write!(f, "CXX_FAST_TLS"),
            CallConvention::CFGuard_Check => write!(f, "CFGuard_Check"),
            CallConvention::MSP430_INTR => write!(f, "MSP430_INTR"),
            CallConvention::X86_INTR => write!(f, "X86_INTR"),
            CallConvention::AVR_INTR => write!(f, "AVR_INTR"),
            CallConvention::AVR_SIGNAL => write!(f, "AVR_SIGNAL"),
            CallConvention::AVR_BUILTIN => write!(f, "AVR_BUILTIN"),
            CallConvention::AMDGPU_VS => write!(f, "AMDGPU_VS"),
            CallConvention::AMDGPU_GS => write!(f, "AMDGPU_GS"),
            CallConvention::AMDGPU_PS => write!(f, "AMDGPU_PS"),
            CallConvention::AMDGPU_CS => write!(f, "AMDGPU_CS"),
            CallConvention::AMDGPU_HS => write!(f, "AMDGPU_HS"),
            CallConvention::MSP430_BUILTIN => write!(f, "MSP430_BUILTIN"),
            CallConvention::AMDGPU_LS => write!(f, "AMDGPU_LS"),
            CallConvention::AMDGPU_ES => write!(f, "AMDGPU_ES"),
            CallConvention::WASM_EmscriptenInvoke => write!(f, "WASM_EmscriptenInvoke"),
            CallConvention::M68k_INTR => write!(f, "M68k_INTR"),
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0 => {
                write!(f, "AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0")
            }
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2 => {
                write!(f, "AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2")
            }
            CallConvention::AMDGPU_CS_Chain => write!(f, "AMDGPU_CS_Chain"),
            CallConvention::AMDGPU_CS_ChainPreserve => write!(f, "AMDGPU_CS_ChainPreserve"),
            CallConvention::M68k_RTD => write!(f, "M68k_RTD"),
            CallConvention::ARM64EC_Thunk_X64 => write!(f, "ARM64EC_Thunk_X64"),
            CallConvention::ARM64EC_Thunk_Native => write!(f, "ARM64EC_Thunk_Native"),
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1 => {
                write!(f, "AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1")
            }
            CallConvention::RISCV_VLSCall_32 => write!(f, "RISCV_VLSCall_32"),
            CallConvention::RISCV_VLSCall_64 => write!(f, "RISCV_VLSCall_64"),
            CallConvention::RISCV_VLSCall_128 => write!(f, "RISCV_VLSCall_128"),
            CallConvention::RISCV_VLSCall_256 => write!(f, "RISCV_VLSCall_256"),
            CallConvention::RISCV_VLSCall_512 => write!(f, "RISCV_VLSCall_512"),
            CallConvention::RISCV_VLSCall_1024 => write!(f, "RISCV_VLSCall_1024"),
            CallConvention::RISCV_VLSCall_2048 => write!(f, "RISCV_VLSCall_2048"),
            CallConvention::RISCV_VLSCall_4096 => write!(f, "RISCV_VLSCall_4096"),
            CallConvention::RISCV_VLSCall_8192 => write!(f, "RISCV_VLSCall_8192"),
            CallConvention::RISCV_VLSCall_16384 => write!(f, "RISCV_VLSCall_16384"),
            CallConvention::RISCV_VLSCall_32768 => write!(f, "RISCV_VLSCall_32768"),
            CallConvention::RISCV_VLSCall_65536 => write!(f, "RISCV_VLSCall_65536"),
            CallConvention::AMDGPU_Gfx_WholeWave => write!(f, "AMDGPU_Gfx_WholeWave"),
            CallConvention::CHERIoT_CompartmentCall => write!(f, "CHERIoT_CompartmentCall"),
            CallConvention::CHERIoT_CompartmentCallee => write!(f, "CHERIoT_CompartmentCallee"),
            CallConvention::CHERIoT_LibraryCall => write!(f, "CHERIoT_LibraryCall"),
            CallConvention::SPIR_FUNC => write!(f, "SPIR_FUNC"),
            CallConvention::SPIR_KERNEL => write!(f, "SPIR_KERNEL"),
            CallConvention::Intel_OCL_BI => write!(f, "Intel_OCL_BI"),
        }
    }
}

impl AssemblerFunctionExtensions for str {
    #[inline]
    fn as_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect {
        match syntax {
            "Intel" => InlineAsmDialect::Intel,
            "AT&T" => InlineAsmDialect::ATT,

            _ => InlineAsmDialect::ATT,
        }
    }
}

impl LLVMLinkageExtensions for Linkage {
    fn fmt(&self) -> &'static str {
        match self {
            Linkage::Appending => "Appending",
            Linkage::Common => "Common",
            Linkage::AvailableExternally => "AvailableExternally",
            Linkage::External => "External",
            Linkage::ExternalWeak => "ExternalWeak",
            Linkage::Internal => "Internal",
            Linkage::LinkOnceAny => "LinkOnceAny",
            Linkage::LinkOnceODR => "LinkOnceODR",
            Linkage::LinkOnceODRAutoHide => "LinkOnceODRAutoHide",
            Linkage::Private => "Private",
            Linkage::WeakAny => "WeakAny",
            Linkage::WeakODR => "WeakODR",
            Linkage::DLLExport => "DLLExport",
            Linkage::DLLImport => "DLLImport",
            Linkage::Ghost => "Ghost",
            Linkage::LinkerPrivate => "LinkerPrivate",
            Linkage::LinkerPrivateWeak => "LinkerPrivateWeak",
        }
    }
}
