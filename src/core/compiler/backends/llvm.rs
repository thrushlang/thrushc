use inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple};

use crate::core::compiler::{
    jit::JITConfiguration, linking::LinkingCompilersConfiguration, options::ThrushOptimization,
    passes::LLVMModificatorPasses,
};

#[derive(Debug)]
pub struct LLVMBackend {
    target_cpu: String,
    target_triple: TargetTriple,
    optimization: ThrushOptimization,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    modificator_passes: Vec<LLVMModificatorPasses>,
    opt_passes: String,
    linking_compilers_config: LinkingCompilersConfiguration,
    jit_config: Option<JITConfiguration>,
}

impl LLVMBackend {
    pub fn new() -> Self {
        Self {
            target_cpu: String::with_capacity(100),
            target_triple: TargetMachine::get_default_triple(),
            optimization: ThrushOptimization::None,
            reloc_mode: RelocMode::PIC,
            code_model: CodeModel::Default,
            modificator_passes: Vec::with_capacity(10),
            opt_passes: String::with_capacity(100),
            linking_compilers_config: LinkingCompilersConfiguration::new(),
            jit_config: None,
        }
    }

    #[inline]
    pub fn get_reloc_mode(&self) -> RelocMode {
        self.reloc_mode
    }

    #[inline]
    pub fn get_code_model(&self) -> CodeModel {
        self.code_model
    }

    #[inline]
    pub fn get_target_triple(&self) -> &TargetTriple {
        &self.target_triple
    }

    #[inline]
    pub fn get_optimization(&self) -> ThrushOptimization {
        self.optimization
    }

    #[inline]
    pub fn get_target_cpu(&self) -> &str {
        self.target_cpu.as_str()
    }

    #[inline]
    pub fn get_opt_passes(&self) -> &str {
        self.opt_passes.as_str()
    }

    #[inline]
    pub fn get_modificator_passes(&self) -> &[LLVMModificatorPasses] {
        &self.modificator_passes
    }

    #[inline]
    pub fn get_linking_compilers_configuration(&self) -> &LinkingCompilersConfiguration {
        &self.linking_compilers_config
    }

    #[inline]
    pub fn get_mut_linking_compilers_configuration(
        &mut self,
    ) -> &mut LinkingCompilersConfiguration {
        &mut self.linking_compilers_config
    }

    #[inline]
    pub fn get_jit_config(&self) -> Option<&JITConfiguration> {
        self.jit_config.as_ref()
    }

    #[inline]
    pub fn get_mut_jit_config(&mut self) -> Option<&mut JITConfiguration> {
        self.jit_config.as_mut()
    }
}

impl LLVMBackend {
    #[inline]
    pub fn set_optimization(&mut self, opt: ThrushOptimization) {
        self.optimization = opt;
    }

    #[inline]
    pub fn set_reloc_mode(&mut self, reloc_mode: RelocMode) {
        self.reloc_mode = reloc_mode;
    }

    #[inline]
    pub fn set_code_model(&mut self, code_model: CodeModel) {
        self.code_model = code_model;
    }

    #[inline]
    pub fn set_target_triple(&mut self, target_triple: TargetTriple) {
        self.target_triple = target_triple;
    }

    #[inline]
    pub fn set_target_cpu(&mut self, target_cpu: String) {
        self.target_cpu = target_cpu;
    }

    #[inline]
    pub fn set_opt_passes(&mut self, opt_passes: String) {
        self.opt_passes = opt_passes;
    }

    #[inline]
    pub fn set_modificator_passes(&mut self, modificator_passes: Vec<LLVMModificatorPasses>) {
        self.modificator_passes = modificator_passes;
    }

    #[inline]
    pub fn set_jit_config(&mut self, jit: JITConfiguration) {
        self.jit_config = Some(jit);
    }
}
