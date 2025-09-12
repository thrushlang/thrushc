pub mod cpu;
pub mod flavors;
pub mod target;
pub mod targets;

use inkwell::targets::{CodeModel, RelocMode, TargetMachine};

use crate::core::compiler::{
    backends::llvm::{cpu::LLVMTargetCPU, target::LLVMTarget},
    options::ThrushOptimization,
    passes::LLVMModificatorPasses,
};

#[derive(Debug)]
pub struct LLVMBackend {
    target: LLVMTarget,
    target_cpu: LLVMTargetCPU,

    optimization: ThrushOptimization,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    modificator_passes: Vec<LLVMModificatorPasses>,
    opt_passes: String,
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
    pub fn set_opt_passes(&mut self, opt_passes: String) {
        self.opt_passes = opt_passes;
    }

    #[inline]
    pub fn set_modificator_passes(&mut self, modificator_passes: Vec<LLVMModificatorPasses>) {
        self.modificator_passes = modificator_passes;
    }
}
