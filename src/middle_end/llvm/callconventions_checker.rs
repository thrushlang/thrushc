use crate::back_end::llvm_codegen::helpertypes::traits::LLVMAttributesExtensions;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

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
            diagnostician: Diagnostician::new(file, options),
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
        let llvm_attributes: crate::back_end::llvm_codegen::helpertypes::repr::LLVMAttributes =
            attributes.as_llvm_attributes();
        let llvm_backend: &LLVMBackend = self.get_compiler_options().get_llvm_backend_options();

        let triple: (String, String, String, String) =
            llvm_backend.get_target().dissamble_target_triple();

        match applicant {
            CallConventionAplicant::Function => {
                if let Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Convention(
                    call_conv,
                )) = llvm_attributes.get_attr(
                    crate::back_end::llvm_codegen::attributes::LLVMAttributeComparator::Convention,
                ) {
                    if let Some(ThrushAttribute::Convention(_, span)) =
                        attributes.get_attr(ThrushAttributeComparator::Convention)
                    {
                        self.analyze_calling_convention(triple, call_conv, applicant, span);
                    }
                }
            }
            CallConventionAplicant::Instrinsic => {
                if let Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Convention(
                    call_conv,
                )) = llvm_attributes.get_attr(
                    crate::back_end::llvm_codegen::attributes::LLVMAttributeComparator::Convention,
                ) {
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
        call_conv: crate::back_end::llvm_codegen::callconventions::CallConvention,
        applicant: CallConventionAplicant,
        span: Span,
    ) {
        const X86_64_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_StdCall,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_FastCall,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_ThisCall,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_64_SysV,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_INTR,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_VectorCall,
            crate::back_end::llvm_codegen::callconventions::CallConvention::X86_RegCall,
        ];

        const ARM_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::ARM_AAPCS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::ARM_AAPCS_VFP,
            crate::back_end::llvm_codegen::callconventions::CallConvention::ARM_APCS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::ARM64EC_Thunk_Native,
            crate::back_end::llvm_codegen::callconventions::CallConvention::ARM64EC_Thunk_X64,
        ];

        const RISCV_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_1024,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_128,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_16384,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_2048,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_256,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_32,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_32768,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_4096,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_512,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_64,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_65536,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VLSCall_8192,
            crate::back_end::llvm_codegen::callconventions::CallConvention::RISCV_VectorCall,
        ];

        const AARCH64_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AArch64_SVE_VectorCall,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AArch64_VectorCall,
        ];

        const AMDGPU_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_CS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_CS_Chain,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_CS_ChainPreserve,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_ES,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_GS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_Gfx,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_Gfx_WholeWave,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_HS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_KERNEL,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_LS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_PS,
            crate::back_end::llvm_codegen::callconventions::CallConvention::AMDGPU_VS,
        ];

        const WASM_CALL_CONVENTIONS:
            &[crate::back_end::llvm_codegen::callconventions::CallConvention] = &[
            crate::back_end::llvm_codegen::callconventions::CallConvention::WASM_EmscriptenInvoke,
        ];

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
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
                                None,
                                span,
                            ));
                        }
                    }
                    arch if arch.contains("arm") => {
                        if !ARM_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
                                None,
                                span,
                            ));
                        }
                    }
                    arch if arch.contains("riscv") => {
                        if !RISCV_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
                                None,
                                span,
                            ));
                        }
                    }
                    arch if arch.contains("aarch64") => {
                        if !AARCH64_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
                                None,
                                span,
                            ));
                        }
                    }
                    arch if arch.starts_with("amd") => {
                        if !AMDGPU_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
                                None,
                                span,
                            ));
                        }
                    }

                    arch if arch.contains("wasm") => {
                        if !WASM_CALL_CONVENTIONS.contains(&call_conv) {
                            self.add_error(CompilationIssue::Error(
                                CompilationIssueCode::E0024,
                                format!("This calling convention is not supported on the '{}' current target architecture. Select another one or change the target architecture.", arch),
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
