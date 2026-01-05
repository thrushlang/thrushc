use thrushc_ast::Ast;
use thrushc_attributes::{
    ThrushAttribute, ThrushAttributeComparator, ThrushAttributes,
    traits::ThrushAttributesExtensions,
};
use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_llvm_attributes::{
    LLVMAttribute, LLVMAttributeComparator, LLVMAttributes, traits::LLVMAttributesExtensions,
};
use thrushc_llvm_callconventions::LLVMCallConvention;
use thrushc_options::{CompilationUnit, CompilerOptions, backends::llvm::LLVMBackend};
use thrushc_span::Span;

#[derive(Debug)]
pub struct LLVMCallConventionsChecker<'call_conv_checker> {
    ast: &'call_conv_checker [Ast<'call_conv_checker>],
    options: &'call_conv_checker CompilerOptions,
    errors: Vec<CompilationIssue>,
    diagnostician: Diagnostician,
}

#[derive(Debug, Clone, Copy)]
pub enum LLVMCallConventionAplicant {
    Function,
    Instrinsic,
}

impl<'call_conv_checker> LLVMCallConventionsChecker<'call_conv_checker> {
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

impl<'call_conv_checker> LLVMCallConventionsChecker<'call_conv_checker> {
    pub fn analyze(&mut self) -> bool {
        for node in self.ast.iter() {
            self.visit_node(node);
        }

        self.verify()
    }
}

impl LLVMCallConventionsChecker<'_> {
    fn visit_node(&mut self, node: &Ast) {
        match node {
            Ast::Function { attributes, .. } => {
                self.analyze_applicant(attributes, LLVMCallConventionAplicant::Function);
            }
            Ast::Intrinsic { attributes, .. } => {
                self.analyze_applicant(attributes, LLVMCallConventionAplicant::Instrinsic);
            }

            _ => (),
        }
    }
}

impl LLVMCallConventionsChecker<'_> {
    fn analyze_applicant(
        &mut self,
        attributes: &ThrushAttributes,
        applicant: LLVMCallConventionAplicant,
    ) {
        let llvm_attributes: LLVMAttributes =
            thrushc_llvm_attributes::into_llvm_attributes(attributes);
        let llvm_backend: &LLVMBackend = self.get_compiler_options().get_llvm_backend_options();

        let triple: (String, String, String, String) =
            llvm_backend.get_target().dissamble_target_triple();

        match applicant {
            LLVMCallConventionAplicant::Function => {
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
            LLVMCallConventionAplicant::Instrinsic => {
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

impl LLVMCallConventionsChecker<'_> {
    fn analyze_calling_convention(
        &mut self,
        target_triple: (String, String, String, String),
        call_conv: LLVMCallConvention,
        applicant: LLVMCallConventionAplicant,
        span: Span,
    ) {
        const X86_64_CALL_CONVENTIONS: &[LLVMCallConvention] = &[
            LLVMCallConvention::X86_StdCall,
            LLVMCallConvention::X86_FastCall,
            LLVMCallConvention::X86_ThisCall,
            LLVMCallConvention::X86_64_SysV,
            LLVMCallConvention::X86_INTR,
            LLVMCallConvention::X86_VectorCall,
            LLVMCallConvention::X86_RegCall,
        ];

        const ARM_CALL_CONVENTIONS: &[LLVMCallConvention] = &[
            LLVMCallConvention::ARM_AAPCS,
            LLVMCallConvention::ARM_AAPCS_VFP,
            LLVMCallConvention::ARM_APCS,
            LLVMCallConvention::ARM64EC_Thunk_Native,
            LLVMCallConvention::ARM64EC_Thunk_X64,
        ];

        const RISCV_CALL_CONVENTIONS: &[LLVMCallConvention] = &[
            LLVMCallConvention::RISCV_VLSCall_1024,
            LLVMCallConvention::RISCV_VLSCall_128,
            LLVMCallConvention::RISCV_VLSCall_16384,
            LLVMCallConvention::RISCV_VLSCall_2048,
            LLVMCallConvention::RISCV_VLSCall_256,
            LLVMCallConvention::RISCV_VLSCall_32,
            LLVMCallConvention::RISCV_VLSCall_32768,
            LLVMCallConvention::RISCV_VLSCall_4096,
            LLVMCallConvention::RISCV_VLSCall_512,
            LLVMCallConvention::RISCV_VLSCall_64,
            LLVMCallConvention::RISCV_VLSCall_65536,
            LLVMCallConvention::RISCV_VLSCall_8192,
            LLVMCallConvention::RISCV_VectorCall,
        ];

        const AARCH64_CALL_CONVENTIONS: &[LLVMCallConvention] = &[
            LLVMCallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X0,
            LLVMCallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X1,
            LLVMCallConvention::AArch64_SME_ABI_Support_Routines_PreserveMost_From_X2,
            LLVMCallConvention::AArch64_SVE_VectorCall,
            LLVMCallConvention::AArch64_VectorCall,
        ];

        const AMDGPU_CALL_CONVENTIONS: &[LLVMCallConvention] = &[
            LLVMCallConvention::AMDGPU_CS,
            LLVMCallConvention::AMDGPU_CS_Chain,
            LLVMCallConvention::AMDGPU_CS_ChainPreserve,
            LLVMCallConvention::AMDGPU_ES,
            LLVMCallConvention::AMDGPU_GS,
            LLVMCallConvention::AMDGPU_Gfx,
            LLVMCallConvention::AMDGPU_Gfx_WholeWave,
            LLVMCallConvention::AMDGPU_HS,
            LLVMCallConvention::AMDGPU_KERNEL,
            LLVMCallConvention::AMDGPU_LS,
            LLVMCallConvention::AMDGPU_PS,
            LLVMCallConvention::AMDGPU_VS,
        ];

        const WASM_CALL_CONVENTIONS: &[LLVMCallConvention] =
            &[LLVMCallConvention::WASM_EmscriptenInvoke];

        let lower_arch: String = target_triple.0.to_lowercase();
        let arch: &str = lower_arch.trim();

        match applicant {
            LLVMCallConventionAplicant::Function | LLVMCallConventionAplicant::Instrinsic
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

impl LLVMCallConventionsChecker<'_> {
    #[inline]
    fn get_compiler_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl LLVMCallConventionsChecker<'_> {
    #[inline]
    fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl LLVMCallConventionsChecker<'_> {
    #[inline]
    fn verify(&mut self) -> bool {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, thrushc_logging::LoggingType::Error);
            });

            true
        } else {
            false
        }
    }
}
