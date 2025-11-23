pub mod cpu;
pub mod flavors;
pub mod info;
pub mod jit;
pub mod passes;
pub mod target;
pub mod targets;
pub mod utils;

use inkwell::targets::{CodeModel, RelocMode, TargetMachine};

use crate::core::compiler::backends::llvm::cpu::LLVMTargetCPU;
use crate::core::compiler::backends::llvm::jit::JITConfiguration;
use crate::core::compiler::backends::llvm::passes::LLVMModificatorPasses;
use crate::core::compiler::backends::llvm::target::LLVMTarget;

use crate::core::compiler::options::ThrushOptimization;

#[derive(Debug)]
pub struct LLVMBackend {
    target: LLVMTarget,
    target_cpu: LLVMTargetCPU,

    optimization: ThrushOptimization,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    modificator_passes: Vec<LLVMModificatorPasses>,
    opt_passes: String,
    omit_frame_pointer: bool,
    omit_uwtable: bool,

    use_jit: bool,
    jit_config: JITConfiguration,
}

impl LLVMBackend {
    pub fn new() -> Self {
        let arch: String = TargetMachine::get_default_triple()
            .as_str()
            .to_string_lossy()
            .split("-")
            .collect::<Vec<_>>()
            .first()
            .map_or("generic", |v| v)
            .to_string();

        Self {
            target: LLVMTarget {
                arch,
                target_triple: TargetMachine::get_default_triple(),
            },

            target_cpu: LLVMTargetCPU {
                target_cpu: TargetMachine::get_host_cpu_name().to_string(),
                target_cpu_feautures: TargetMachine::get_host_cpu_features().to_string(),
            },

            optimization: ThrushOptimization::None,
            reloc_mode: RelocMode::PIC,
            code_model: CodeModel::Default,
            modificator_passes: Vec::with_capacity(10),
            opt_passes: String::with_capacity(100),
            omit_frame_pointer: false,
            omit_uwtable: false,
            use_jit: false,
            jit_config: JITConfiguration::new(),
        }
    }
}

impl LLVMBackend {
    #[inline]
    pub fn get_reloc_mode(&self) -> RelocMode {
        self.reloc_mode
    }

    #[inline]
    pub fn get_code_model(&self) -> CodeModel {
        self.code_model
    }

    #[inline]
    pub fn get_optimization(&self) -> ThrushOptimization {
        self.optimization
    }

    #[inline]
    pub fn get_target(&self) -> &LLVMTarget {
        &self.target
    }

    #[inline]
    pub fn get_target_cpu(&self) -> &LLVMTargetCPU {
        &self.target_cpu
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
    pub fn get_jit_config(&self) -> &JITConfiguration {
        &self.jit_config
    }

    #[inline]
    pub fn omit_frame_pointer(&self) -> bool {
        self.omit_frame_pointer
    }

    #[inline]
    pub fn omit_uwtable(&self) -> bool {
        self.omit_uwtable
    }

    #[inline]
    pub fn is_jit(&self) -> bool {
        self.use_jit
    }
}

impl LLVMBackend {
    #[inline]
    pub fn get_mut_target(&mut self) -> &mut LLVMTarget {
        &mut self.target
    }

    #[inline]
    pub fn get_mut_target_cpu(&mut self) -> &mut LLVMTargetCPU {
        &mut self.target_cpu
    }

    #[inline]
    pub fn get_mut_jit_config(&mut self) -> &mut JITConfiguration {
        &mut self.jit_config
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
    pub fn set_omit_frame_pointer(&mut self) {
        self.omit_frame_pointer = true;
    }

    #[inline]
    pub fn set_omit_uwtable(&mut self) {
        self.omit_uwtable = true;
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
    pub fn set_jit(&mut self, use_jit: bool) {
        self.use_jit = use_jit;
    }
}
