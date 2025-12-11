use crate::back_end::llvm_codegen::attributes::{LLVMAttribute, LLVMAttributeComparator};
use crate::back_end::llvm_codegen::callconventions::CallConvention;
use crate::back_end::llvm_codegen::types::repr::LLVMAttributes;
use crate::back_end::llvm_codegen::types::traits::LLVMAttributesExtensions;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;
use crate::middle_end::mir::attributes::{
    ThrushAttribute, ThrushAttributeComparator, ThrushAttributes,
};

use crate::front_end::types::ast::Ast;

#[derive(Debug)]
pub struct CallConventionsChecker<'call_conv_checker> {
    ast: &'call_conv_checker [Ast<'call_conv_checker>],
    options: &'call_conv_checker CompilerOptions,
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

#[derive(Debug, Clone, Copy)]
pub enum CallConventionAplicant {
    Function,
    Instrinsic,
}

impl<'call_conv_checker> CallConventionsChecker<'call_conv_checker> {
    #[inline]
    pub fn new(
        ast: &'call_conv_checker [Ast<'call_conv_checker>],
        options: &'call_conv_checker CompilerOptions,
        file: &'call_conv_checker CompilationUnit,
    ) -> Self {
        Self {
            ast,
            options,
            errors: Vec::with_capacity(100),
            diagnostician: Diagnostician::new(file),
        }
    }
}

impl<'call_conv_checker> CallConventionsChecker<'call_conv_checker> {
    pub fn check(&mut self) -> Result<(), ()> {
        for node in self.ast.iter() {
            self.visit_node(node);
        }

        self.verify()?;

        Ok(())
    }
}

impl CallConventionsChecker<'_> {
    fn visit_node(&mut self, node: &Ast) {
        match node {
            Ast::Function { attributes, .. } => {
                self.analyze_applicant(attributes, CallConventionAplicant::Function);
            }
            Ast::Intrinsic { attributes, .. } => {
                self.analyze_applicant(attributes, CallConventionAplicant::Instrinsic);
            }

            _ => (),
        }
    }
}

impl CallConventionsChecker<'_> {
    fn analyze_applicant(
        &mut self,
        attributes: &ThrushAttributes,
        applicant: CallConventionAplicant,
    ) {
        let llvm_attributes: LLVMAttributes = attributes.as_llvm_attributes();
        let llvm_backend: &LLVMBackend = self.get_compiler_options().get_llvm_backend_options();

        let triple: (String, String, String, String) =
            llvm_backend.get_target().dissamble_target_triple();

        match applicant {
            CallConventionAplicant::Function => {
                if let Some(LLVMAttribute::Convention(call_conv)) =
                    llvm_attributes.get_attr(LLVMAttributeComparator::Convention)
                {
                    if let Some(ThrushAttribute::Convention(_, span)) =
                        attributes.get_attr(ThrushAttributeComparator::Convention)
                    {
                        self.analyze_calling_convention(triple, call_conv, applicant, span);
                    }
                }
            }
            CallConventionAplicant::Instrinsic => {
                if let Some(LLVMAttribute::Convention(call_conv)) =
                    llvm_attributes.get_attr(LLVMAttributeComparator::Convention)
                {
                    if let Some(ThrushAttribute::Convention(_, span)) =
                        attributes.get_attr(ThrushAttributeComparator::Convention)
                    {
                        self.analyze_calling_convention(triple, call_conv, applicant, span);
                    }
                }
            }
        }
    }
}

impl CallConventionsChecker<'_> {
    fn analyze_calling_convention(
        &mut self,
        target_triple: (String, String, String, String),
        call_conv: CallConvention,
        applicant: CallConventionAplicant,
        span: Span,
    ) {
        const X86_64_CALL_CONVENTIONS: &[CallConvention] = &[
            CallConvention::X86_StdCall,
            CallConvention::X86_FastCall,
            CallConvention::X86_ThisCall,
            CallConvention::X86_64_SysV,
            CallConvention::X86_INTR,
            CallConvention::X86_VectorCall,
            CallConvention::X86_RegCall,
        ];

        const ARM_CALL_CONVENTIONS: &[CallConvention] = &[
            CallConvention::ARM_AAPCS,
            CallConvention::ARM_AAPCS_VFP,
            CallConvention::ARM_APCS,
            CallConvention::ARM64EC_Thunk_Native,
            CallConvention::ARM64EC_Thunk_X64,
        ];

        const RISCV_CALL_CONVENTIONS: &[CallConvention] = &[
            CallConvention::RISCV_VLSCall_1024,
            CallConvention::RISCV_VLSCall_128,
            CallConvention::RISCV_VLSCall_16384,
            CallConvention::RISCV_VLSCall_2048,
            CallConvention::RISCV_VLSCall_256,
            CallConvention::RISCV_VLSCall_32,
            CallConvention::RISCV_VLSCall_32768,
            CallConvention::RISCV_VLSCall_4096,
            CallConvention::RISCV_VLSCall_512,
            CallConvention::RISCV_VLSCall_64,
            CallConvention::RISCV_VLSCall_65536,
            CallConvention::RISCV_VLSCall_8192,
            CallConvention::RISCV_VectorCall,
        ];

        const AARCH64_CALL_CONVENTIONS: &[CallConvention] = &[
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0,
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1,
            CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2,
            CallConvention::AArch64_SVE_VectorCall,
            CallConvention::AArch64_VectorCall,
        ];

        const AMDGPU_CALL_CONVENTIONS: &[CallConvention] = &[
            CallConvention::AMDGPU_CS,
            CallConvention::AMDGPU_CS_Chain,
            CallConvention::AMDGPU_CS_ChainPreserve,
            CallConvention::AMDGPU_ES,
            CallConvention::AMDGPU_GS,
            CallConvention::AMDGPU_Gfx,
            CallConvention::AMDGPU_Gfx_WholeWave,
            CallConvention::AMDGPU_HS,
            CallConvention::AMDGPU_KERNEL,
            CallConvention::AMDGPU_LS,
            CallConvention::AMDGPU_PS,
            CallConvention::AMDGPU_VS,
        ];

        const WASM_CALL_CONVENTIONS: &[CallConvention] = &[CallConvention::WASM_EmscriptenInvoke];

        let lower_arch: String = target_triple.0.to_lowercase();
        let arch: &str = lower_arch.trim();

        match applicant {
            CallConventionAplicant::Function | CallConventionAplicant::Instrinsic
                if call_conv.is_specific_target_conv() =>
            {
                match arch {
                    arch if arch.contains("x86") => {
                        if !X86_64_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                "Unsupported Call Convention".into(),
                                "This calling convention is not supported on the 'x86_64' target architecture. Select another one or change the target architecture.".into(),
                                None,
                                span,
                            ));
                        }
                    }
                    arch if arch.contains("arm") => {
                        if !ARM_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                            "Unsupported Call Convention".into(),
                            "This calling convention is not supported on the 'arm' target architecture. Select another one or change the target architecture.".into(),
                            None,
                            span,
                        ));
                        }
                    }
                    arch if arch.contains("riscv") => {
                        if !RISCV_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                            "Unsupported Call Convention".into(),
                            "This calling convention is not supported on the 'riscv' target architecture. Select another one or change the target architecture.".into(),
                            None,
                            span,
                        ));
                        }
                    }
                    arch if arch.contains("aarch64") => {
                        if !AARCH64_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                            "Unsupported Call Convention".into(),
                            "This calling convention is not supported on the 'aarch64' target architecture. Select another one or change the target architecture.".into(),
                            None,
                            span,
                        ));
                        }
                    }
                    arch if arch.starts_with("amd") => {
                        if !AMDGPU_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                            "Unsupported Call Convention".into(),
                            "This calling convention is not supported on the 'amd' target architecture. Select another one or change the target architecture.".into(),
                            None,
                            span,
                        ));
                        }
                    }

                    arch if arch.contains("wasm") => {
                        if !WASM_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                            "Unsupported Call Convention".into(),
                            "This calling convention is not supported on the 'wasm' target architecture. Select another one or change the target architecture.".into(),
                            None,
                            span,
                        ));
                        }
                    }

                    _ => (),
                }
            }

            _ => (),
        }
    }
}

impl CallConventionsChecker<'_> {
    #[inline]
    fn get_compiler_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl CallConventionsChecker<'_> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl CallConventionsChecker<'_> {
    #[inline]
    fn verify(&mut self) -> Result<(), ()> {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return Err(());
        }

        Ok(())
    }
}
